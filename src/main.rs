use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info, warn};

mod boot;
mod config;
mod errors;
mod models;
mod services;

use config::AppConfig;
use services::{DatabaseService, KafkaConsumerService, MessageConsumer, MessageProcessor};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging early
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    info!(
        "🚀 Iniciando Siscom Consumer Rust v{}",
        env!("CARGO_PKG_VERSION")
    );

    boot::print_banner();

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
    info!("✅ Configuración cargada y validada");

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
    message_consumer: Box<dyn MessageConsumer>,
    database: Arc<DatabaseService>,
    message_processor: MessageProcessor,
    message_receiver: tokio::sync::mpsc::UnboundedReceiver<models::DeviceMessage>,
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

    // Inicializar Kafka consumer
    info!("📡 Inicializando Kafka consumer...");
    let message_consumer: Box<dyn MessageConsumer> =
        Box::new(KafkaConsumerService::new(&config.broker)?);

    // Iniciar el consumo y obtener el receiver
    let message_receiver = message_consumer.start_consuming().await?;

    // Inicializar el procesador de mensajes
    let message_processor = MessageProcessor::new(
        database.clone(),
        config.processing.batch_processing_size,
        5000, // 5 segundos de intervalo de flush
    );

    Ok(Services {
        message_consumer,
        database,
        message_processor,
        message_receiver,
    })
}

/// Loop principal de procesamiento
async fn start_processing_loop(
    services: Services,
    shutdown_signal: tokio::sync::oneshot::Receiver<()>,
) -> Result<()> {
    info!("🚀 Iniciando loop principal de procesamiento...");

    // Start message consumer (spawns its own background task internally)
    let _consumer_receiver = services.message_consumer.start_consuming().await?;
    // Note: We ignore this receiver since we already have one from initialization

    // Start message processor
    let processor = services.message_processor.clone();
    let message_receiver = services.message_receiver;
    let processor_task = tokio::spawn(async move {
        if let Err(e) = processor.start_processing(message_receiver).await {
            error!("Error en message processor: {}", e);
        }
    });

    // Health check task
    let health_db = services.database.clone();
    let health_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let db_health = health_db.health_check().await.unwrap_or(false);
            if !db_health {
                warn!("⚠️ Base de datos no está saludable");
            } else {
                info!("💚 Base de datos saludable");
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
                "📊 Estadísticas - DB Buffer: {}, Batch Size: {}",
                stats.db_buffer_size, stats.batch_size
            );
        }
    });

    // Wait for shutdown signal or task completion
    tokio::select! {
        _ = shutdown_signal => {
            info!("🔔 Señal de shutdown recibida");
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

    // Disconnect message consumer
    if let Err(e) = services.message_consumer.disconnect().await {
        error!("Error desconectando message consumer: {}", e);
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
