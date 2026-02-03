use anyhow::Result;
use config::ConfigError;
use serde::{Deserialize, Serialize};
use std::fs;

/// Tipos de broker soportados
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BrokerType {
    #[serde(rename = "kafka")]
    Kafka,
}

/// Configuración unificada para el broker (Kafka)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerConfig {
    pub broker_type: BrokerType,
    pub host: String,
    pub topic: String,
    pub group_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub broker: BrokerConfig,
    pub database: DatabaseConfig,
    pub processing: ProcessingConfig,
    pub logging: LoggingConfig,
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
    /// Carga la configuración desde archivo .env y variables de entorno
    pub fn load() -> Result<Self, ConfigError> {
        use std::env;

        // Cargar variables del archivo .env si existe
        if let Ok(content) = fs::read_to_string(".env") {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    env::set_var(key.trim(), value.trim().trim_matches('"'));
                }
            }
        }

        // Broker Configuration
        let broker_type_str = env::var("BROKER_TYPE").unwrap_or_else(|_| "kafka".to_string());
        let broker_type = match broker_type_str.to_lowercase().as_str() {
            "kafka" | "redpanda" => BrokerType::Kafka,
            _ => {
                eprintln!(
                    "⚠️ BROKER_TYPE '{}' no reconocido, usando 'kafka' por defecto",
                    broker_type_str
                );
                BrokerType::Kafka
            }
        };

        let broker_host = env::var("BROKER_HOST").unwrap_or_else(|_| "127.0.0.1:9092".to_string());

        let broker_topic =
            env::var("BROKER_TOPIC").unwrap_or_else(|_| "siscom-messages".to_string());

        let broker_group_id =
            env::var("BROKER_GROUP_ID").unwrap_or_else(|_| "siscom-consumer-group".to_string());

        // Kafka-specific configuration (usados solo si broker_type es Kafka)
        // Database Configuration
        let db_host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let db_port = env::var("DB_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse::<u16>()
            .unwrap_or(5432);
        let db_database = env::var("DB_DATABASE").unwrap_or_else(|_| "tracking".to_string());
        let db_username = env::var("DB_USERNAME").unwrap_or_else(|_| "user".to_string());
        let db_password = env::var("DB_PASSWORD").unwrap_or_else(|_| "pass".to_string());
        let db_max_connections = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".to_string())
            .parse::<u32>()
            .unwrap_or(20);
        let db_min_connections = env::var("DB_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .unwrap_or(5);
        let db_connection_timeout_secs = env::var("DB_CONNECTION_TIMEOUT_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<u64>()
            .unwrap_or(30);
        let db_idle_timeout_secs = env::var("DB_IDLE_TIMEOUT_SECS")
            .unwrap_or_else(|_| "600".to_string())
            .parse::<u64>()
            .unwrap_or(600);

        // Processing Configuration
        let processing_worker_threads = env::var("PROCESSING_WORKER_THREADS")
            .unwrap_or_else(|_| "4".to_string())
            .parse::<usize>()
            .unwrap_or(4);
        let processing_message_buffer_size = env::var("PROCESSING_MESSAGE_BUFFER_SIZE")
            .unwrap_or_else(|_| "10000".to_string())
            .parse::<usize>()
            .unwrap_or(10000);
        let processing_batch_size = env::var("PROCESSING_BATCH_PROCESSING_SIZE")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<usize>()
            .unwrap_or(100);
        let processing_max_parallel = env::var("PROCESSING_MAX_PARALLEL_DEVICES")
            .unwrap_or_else(|_| "50".to_string())
            .parse::<usize>()
            .unwrap_or(50);

        // Logging Configuration
        let logging_level = env::var("RUST_LOG")
            .or_else(|_| env::var("LOGGING_LEVEL"))
            .unwrap_or_else(|_| "info".to_string());
        let logging_file_path = env::var("LOGGING_FILE_PATH").ok();
        let logging_max_file_size_mb = env::var("LOGGING_MAX_FILE_SIZE_MB")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u64>()
            .unwrap_or(100);
        let logging_max_files = env::var("LOGGING_MAX_FILES")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .unwrap_or(10);
        let logging_json_format = env::var("LOGGING_JSON_FORMAT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        Ok(Self {
            broker: BrokerConfig {
                broker_type,
                host: broker_host,
                topic: broker_topic,
                group_id: broker_group_id,
            },
            database: DatabaseConfig {
                host: db_host,
                port: db_port,
                database: db_database,
                username: db_username,
                password: db_password,
                max_connections: db_max_connections,
                min_connections: db_min_connections,
                connection_timeout_secs: db_connection_timeout_secs,
                idle_timeout_secs: db_idle_timeout_secs,
            },
            processing: ProcessingConfig {
                worker_threads: processing_worker_threads,
                message_buffer_size: processing_message_buffer_size,
                batch_processing_size: processing_batch_size,
                max_parallel_devices: processing_max_parallel,
            },
            logging: LoggingConfig {
                level: logging_level,
                file_path: logging_file_path,
                max_file_size_mb: logging_max_file_size_mb,
                max_files: logging_max_files,
                json_format: logging_json_format,
            },
        })
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
        // Validar configuración del broker
        if self.broker.host.is_empty() {
            return Err(anyhow::anyhow!("Broker host no puede estar vacío"));
        }

        if self.broker.topic.is_empty() {
            return Err(anyhow::anyhow!("Broker topic no puede estar vacío"));
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
            broker: BrokerConfig {
                broker_type: BrokerType::Kafka,
                host: "127.0.0.1:9092".to_string(),
                topic: "siscom-messages".to_string(),
                group_id: "siscom-consumer-group".to_string(),
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
            broker: BrokerConfigSafe {
                broker_type: "kafka".to_string(),
                host: self.broker.host.clone(),
                topic: self.broker.topic.clone(),
                group_id: self.broker.group_id.clone(),
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
    pub broker: BrokerConfigSafe,
    pub database: DatabaseConfigSafe,
    pub processing: ProcessingConfig,
}

#[derive(Debug, Serialize)]
pub struct BrokerConfigSafe {
    pub broker_type: String,
    pub host: String,
    pub topic: String,
    pub group_id: String,
}

#[derive(Debug, Serialize)]
pub struct DatabaseConfigSafe {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub max_connections: u32,
}

// Módulo para incluir el código generado de protobuf
// Este se generará automáticamente con build.rs
#[path = "siscom.v1.rs"]
pub mod siscom;
