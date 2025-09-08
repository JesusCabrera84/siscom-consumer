use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info, warn};

mod config;
mod errors;
mod models;
mod services;

use config::AppConfig;
use services::{DatabaseService, KafkaProducerService, MessageProcessor, MqttConsumerService};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging early
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    info!(
        "🚀 Iniciando Tracking Consumer Rust v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Load configuration
    let config = match AppConfig::load() {
        Ok(config) => {
            config.validate()?;
            info!("✅ Configuración cargada y validada");
            info!("📋 Config: {:#?}", config.display_safe());
            config
        }
        Err(e) => {
            error!("❌ Error cargando configuración: {}", e);
            warn!("🔄 Usando configuración por defecto de desarrollo");
            AppConfig::default_dev()
        }
    };

    // Setup graceful shutdown
    let shutdown_signal = setup_shutdown_handler();

    // Initialize services
    let services = match initialize_services(&config).await {
        Ok(services) => services,
        Err(e) => {
            error!("❌ Error inicializando servicios: {}", e);
            return Err(e);
        }
    };

    info!("✅ Todos los servicios inicializados correctamente");

    // Start the main processing loop
    let processing_result = start_processing_loop(services, shutdown_signal).await;

    match processing_result {
        Ok(_) => info!("✅ Aplicación terminada correctamente"),
        Err(e) => error!("❌ Error en loop principal: {}", e),
    }

    info!("🛑 Tracking Consumer terminado");
    Ok(())
}

/// Estructura que contiene todos los servicios inicializados
struct Services {
    mqtt_consumer: MqttConsumerService,
    database: Arc<DatabaseService>,
    kafka_producer: Arc<KafkaProducerService>,
    message_processor: MessageProcessor,
    mqtt_receiver: tokio::sync::mpsc::UnboundedReceiver<models::SuntechMessage>,
}

/// Inicializa todos los servicios necesarios
async fn initialize_services(config: &AppConfig) -> Result<Services> {
    info!("🔧 Inicializando servicios...");

    // Initialize database service
    info!("🗄️ Conectando a PostgreSQL...");
    let database = Arc::new(
        DatabaseService::new(
            &config.database_url(),
            config.database.max_connections,
            config.processing.batch_processing_size,
        )
        .await?,
    );

    // Initialize Kafka producer
    info!("📤 Configurando Kafka producer...");
    let kafka_producer = Arc::new(KafkaProducerService::new(
        &config.kafka.brokers,
        config.kafka.position_topic.clone(),
        config.kafka.notifications_topic.clone(),
        config.kafka.batch_size,
        config.kafka.compression.as_deref(),
        config.kafka.retries,
    )?);

    // Initialize MQTT consumer
    info!("📥 Configurando MQTT consumer...");
    let (mqtt_consumer, mqtt_receiver) = MqttConsumerService::new(
        &config.mqtt.broker,
        config.mqtt.port,
        &config.mqtt.topic,
        config.mqtt.username.as_deref(),
        config.mqtt.password.as_deref(),
        &config.mqtt.client_id,
        config.mqtt.keep_alive_secs,
        config.mqtt.clean_session,
        config.processing.message_buffer_size,
    )?;

    // Initialize message processor
    info!("⚙️ Configurando procesador de mensajes...");
    let message_processor = MessageProcessor::new(
        database.clone(),
        kafka_producer.clone(),
        config.processing.batch_processing_size,
        config.kafka.batch_timeout_ms,
    );

    Ok(Services {
        mqtt_consumer,
        database,
        kafka_producer,
        message_processor,
        mqtt_receiver,
    })
}

/// Loop principal de procesamiento
async fn start_processing_loop(
    services: Services,
    shutdown_signal: tokio::sync::oneshot::Receiver<()>,
) -> Result<()> {
    info!("🚀 Iniciando loop principal de procesamiento...");

    // Start MQTT consumer in background
    let mqtt_consumer = services.mqtt_consumer.clone();
    let mqtt_task = tokio::spawn(async move {
        if let Err(e) = mqtt_consumer.start_consuming().await {
            error!("Error en MQTT consumer: {}", e);
        }
    });

    // Start message processor
    let processor = services.message_processor.clone();
    let mqtt_receiver = services.mqtt_receiver;
    let processor_task = tokio::spawn(async move {
        if let Err(e) = processor.start_processing(mqtt_receiver).await {
            error!("Error en message processor: {}", e);
        }
    });

    // Health check task
    let health_db = services.database.clone();
    let health_kafka = services.kafka_producer.clone();
    let health_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;

            let db_health = health_db.health_check().await.unwrap_or(false);
            let kafka_health = health_kafka.health_check().await.unwrap_or(false);

            if !db_health {
                warn!("⚠️ Base de datos no está saludable");
            }

            if !kafka_health {
                warn!("⚠️ Kafka no está saludable");
            }

            if db_health && kafka_health {
                info!("💚 Todos los servicios están saludables");
            }
        }
    });

    // Statistics task
    let stats_processor = services.message_processor.clone();
    let stats_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;

            let stats = stats_processor.get_statistics().await;
            info!(
                "📊 Estadísticas - DB Buffer: {}, Kafka Buffer: {}, Batch Size: {}",
                stats.db_buffer_size, stats.kafka_buffer_size, stats.batch_size
            );
        }
    });

    // Wait for shutdown signal or task completion
    tokio::select! {
        _ = shutdown_signal => {
            info!("🔔 Señal de shutdown recibida");
        }
        _ = mqtt_task => {
            warn!("🔌 MQTT task terminado inesperadamente");
        }
        _ = processor_task => {
            warn!("⚙️ Processor task terminado inesperadamente");
        }
        _ = health_task => {
            warn!("💊 Health check task terminado inesperadamente");
        }
        _ = stats_task => {
            warn!("📊 Stats task terminado inesperadamente");
        }
    }

    // Graceful shutdown
    info!("🔄 Iniciando shutdown graceful...");

    // Flush all pending data
    if let Err(e) = services.message_processor.flush_all_buffers().await {
        error!("Error flushing buffers: {}", e);
    }

    // Shutdown Kafka producer
    if let Err(e) = services.kafka_producer.shutdown().await {
        error!("Error cerrando Kafka producer: {}", e);
    }

    // Disconnect MQTT
    if let Err(e) = services.mqtt_consumer.disconnect().await {
        error!("Error desconectando MQTT: {}", e);
    }

    info!("✅ Shutdown completado");
    Ok(())
}

/// Configura el handler para señales de shutdown graceful
fn setup_shutdown_handler() -> tokio::sync::oneshot::Receiver<()> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let mut tx = Some(tx);

        // Handle Ctrl+C
        if let Ok(()) = signal::ctrl_c().await {
            info!("🔔 Ctrl+C recibido");
            if let Some(sender) = tx.take() {
                let _ = sender.send(());
            }
        }
    });

    rx
}
