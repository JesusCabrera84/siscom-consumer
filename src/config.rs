use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mqtt: MqttConfig,
    pub kafka: KafkaConfig,
    pub database: DatabaseConfig,
    pub processing: ProcessingConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub broker: String,
    pub port: u16,
    pub topic: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub client_id: String,
    pub keep_alive_secs: u64,
    pub clean_session: bool,
    pub max_reconnect_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub position_topic: String,
    pub notifications_topic: String,
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
    pub compression: Option<String>,
    pub retries: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_secs: u64,
    pub idle_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    pub poi_radius_meters: f64,
    pub geofence_check_interval_ms: u64,
    pub worker_threads: usize,
    pub message_buffer_size: usize,
    pub batch_processing_size: usize,
    pub max_parallel_devices: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub max_file_size_mb: u64,
    pub max_files: u32,
    pub json_format: bool,
}

impl AppConfig {
    /// Carga la configuración desde archivos y variables de entorno
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Config::builder();

        // Cargar configuración base desde archivo TOML
        let config_path = "config/app.toml";
        if Path::new(config_path).exists() {
            config = config.add_source(File::with_name(config_path));
        }

        // Permitir override con variables de entorno
        config = config.add_source(
            Environment::with_prefix("TRACKING_CONSUMER")
                .separator("_")
                .try_parsing(true)
        );

        let config = config.build()?;
        config.try_deserialize()
    }

    /// Obtiene la URL de conexión a PostgreSQL
    pub fn database_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database
        )
    }

    /// Valida la configuración
    pub fn validate(&self) -> Result<()> {
        // Validar configuración MQTT
        if self.mqtt.broker.is_empty() {
            return Err(anyhow::anyhow!("MQTT broker no puede estar vacío"));
        }

        if self.mqtt.topic.is_empty() {
            return Err(anyhow::anyhow!("MQTT topic no puede estar vacío"));
        }

        // Validar configuración Kafka
        if self.kafka.brokers.is_empty() {
            return Err(anyhow::anyhow!("Kafka brokers no puede estar vacío"));
        }

        if self.kafka.position_topic.is_empty() {
            return Err(anyhow::anyhow!("Kafka position topic no puede estar vacío"));
        }

        // Validar configuración de base de datos
        if self.database.host.is_empty() {
            return Err(anyhow::anyhow!("Database host no puede estar vacío"));
        }

        if self.database.database.is_empty() {
            return Err(anyhow::anyhow!("Database name no puede estar vacío"));
        }

        // Validar configuración de procesamiento
        if self.processing.batch_processing_size == 0 {
            return Err(anyhow::anyhow!("Batch processing size debe ser mayor a 0"));
        }

        if self.processing.worker_threads == 0 {
            return Err(anyhow::anyhow!("Worker threads debe ser mayor a 0"));
        }

        Ok(())
    }

    /// Configuración por defecto para desarrollo
    pub fn default_dev() -> Self {
        Self {
            mqtt: MqttConfig {
                broker: "localhost".to_string(),
                port: 1883,
                topic: "tracking/data".to_string(),
                username: None,
                password: None,
                client_id: "tracking-consumer-rust-dev".to_string(),
                keep_alive_secs: 60,
                clean_session: true,
                max_reconnect_attempts: 10,
            },
            kafka: KafkaConfig {
                brokers: vec!["localhost:9092".to_string()],
                position_topic: "position-topic".to_string(),
                notifications_topic: "notifications-topic".to_string(),
                batch_size: 100,
                batch_timeout_ms: 100,
                compression: Some("snappy".to_string()),
                retries: 3,
            },
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                database: "tracking".to_string(),
                username: "user".to_string(),
                password: "pass".to_string(),
                max_connections: 20,
                min_connections: 5,
                connection_timeout_secs: 30,
                idle_timeout_secs: 600,
            },
            processing: ProcessingConfig {
                poi_radius_meters: 100.0,
                geofence_check_interval_ms: 1000,
                worker_threads: 4,
                message_buffer_size: 10000,
                batch_processing_size: 100,
                max_parallel_devices: 50,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
                max_file_size_mb: 100,
                max_files: 10,
                json_format: true,
            },
        }
    }

    /// Muestra la configuración (ocultando información sensible)
    pub fn display_safe(&self) -> AppConfigSafe {
        AppConfigSafe {
            mqtt: MqttConfigSafe {
                broker: self.mqtt.broker.clone(),
                port: self.mqtt.port,
                topic: self.mqtt.topic.clone(),
                client_id: self.mqtt.client_id.clone(),
                has_credentials: self.mqtt.username.is_some() && self.mqtt.password.is_some(),
            },
            kafka: KafkaConfigSafe {
                brokers: self.kafka.brokers.clone(),
                position_topic: self.kafka.position_topic.clone(),
                notifications_topic: self.kafka.notifications_topic.clone(),
                batch_size: self.kafka.batch_size,
            },
            database: DatabaseConfigSafe {
                host: self.database.host.clone(),
                port: self.database.port,
                database: self.database.database.clone(),
                max_connections: self.database.max_connections,
            },
            processing: self.processing.clone(),
        }
    }
}

/// Versión segura de la configuración para mostrar en logs
#[derive(Debug, Serialize)]
pub struct AppConfigSafe {
    pub mqtt: MqttConfigSafe,
    pub kafka: KafkaConfigSafe,
    pub database: DatabaseConfigSafe,
    pub processing: ProcessingConfig,
}

#[derive(Debug, Serialize)]
pub struct MqttConfigSafe {
    pub broker: String,
    pub port: u16,
    pub topic: String,
    pub client_id: String,
    pub has_credentials: bool,
}

#[derive(Debug, Serialize)]
pub struct KafkaConfigSafe {
    pub brokers: Vec<String>,
    pub position_topic: String,
    pub notifications_topic: String,
    pub batch_size: usize,
}

#[derive(Debug, Serialize)]
pub struct DatabaseConfigSafe {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub max_connections: u32,
}
