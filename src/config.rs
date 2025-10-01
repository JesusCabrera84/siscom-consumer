use anyhow::Result;
use config::ConfigError;
use serde::{Deserialize, Serialize};

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
    pub enabled: bool, // NUEVO: bandera para habilitar/deshabilitar Kafka
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
    /// Carga la configuración solo desde variables de entorno
    pub fn load() -> Result<Self, ConfigError> {
        // Leer variables de entorno directamente sin prefijo
        use std::env;

        // MQTT Configuration
        let _broker_type = env::var("BROKER_TYPE").unwrap_or_else(|_| "mqtt".to_string());

        // Parse BROKER_HOST que puede venir como "host" o "host:port"
        let broker_host_raw = env::var("BROKER_HOST")
            .or_else(|_| env::var("MQTT_BROKER"))
            .unwrap_or_else(|_| "localhost".to_string());

        let (broker_host, broker_port) = if broker_host_raw.contains(':') {
            // Si contiene ':', separar host y puerto
            let parts: Vec<&str> = broker_host_raw.splitn(2, ':').collect();
            let host = parts[0].to_string();
            let port = parts
                .get(1)
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(1883);
            (host, port)
        } else {
            // Si no contiene ':', usar MQTT_PORT separado
            let port = env::var("MQTT_PORT")
                .or_else(|_| env::var("BROKER_PORT"))
                .unwrap_or_else(|_| "1883".to_string())
                .parse::<u16>()
                .unwrap_or(1883);
            (broker_host_raw, port)
        };
        let broker_topic = env::var("BROKER_TOPIC")
            .or_else(|_| env::var("MQTT_TOPIC"))
            .unwrap_or_else(|_| "tracking/data".to_string());

        // Leer credenciales MQTT, convertir strings vacíos en None
        let mqtt_username = env::var("MQTT_USERNAME").ok().and_then(|s| {
            if s.trim().is_empty() {
                None
            } else {
                Some(s)
            }
        });
        let mqtt_password = env::var("MQTT_PASSWORD").ok().and_then(|s| {
            if s.trim().is_empty() {
                None
            } else {
                Some(s)
            }
        });
        let mqtt_client_id =
            env::var("MQTT_CLIENT_ID").unwrap_or_else(|_| "siscom-consumer-rust".to_string());
        let mqtt_keep_alive_secs = env::var("MQTT_KEEP_ALIVE_SECS")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u64>()
            .unwrap_or(60);
        let mqtt_clean_session = env::var("MQTT_CLEAN_SESSION")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        let mqtt_max_reconnect = env::var("MQTT_MAX_RECONNECT_ATTEMPTS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .unwrap_or(10);

        // Kafka Configuration
        let kafka_enabled = env::var("KAFKA_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        let kafka_brokers = env::var("KAFKA_BROKERS")
            .unwrap_or_else(|_| "localhost:9092".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();
        let kafka_position_topic =
            env::var("KAFKA_POSITION_TOPIC").unwrap_or_else(|_| "position-topic".to_string());
        let kafka_notifications_topic = env::var("KAFKA_NOTIFICATIONS_TOPIC")
            .unwrap_or_else(|_| "notifications-topic".to_string());
        let kafka_batch_size = env::var("KAFKA_BATCH_SIZE")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<usize>()
            .unwrap_or(100);
        let kafka_batch_timeout_ms = env::var("KAFKA_BATCH_TIMEOUT_MS")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u64>()
            .unwrap_or(100);
        let kafka_compression = env::var("KAFKA_COMPRESSION").ok();
        let kafka_retries = env::var("KAFKA_RETRIES")
            .unwrap_or_else(|_| "3".to_string())
            .parse::<i32>()
            .unwrap_or(3);

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

        // Log de debug para verificar credenciales (sin mostrar contraseña)
        eprintln!("🔍 Debug MQTT Config:");
        eprintln!("  - BROKER_HOST: {}", broker_host);
        eprintln!("  - BROKER_PORT: {}", broker_port);
        eprintln!(
            "  - MQTT_USERNAME: {}",
            mqtt_username
                .as_ref()
                .map(|_| "[SET]")
                .unwrap_or("[NOT SET]")
        );
        eprintln!(
            "  - MQTT_PASSWORD: {}",
            mqtt_password
                .as_ref()
                .map(|_| "[SET]")
                .unwrap_or("[NOT SET]")
        );
        eprintln!("  - MQTT_CLIENT_ID: {}", mqtt_client_id);

        Ok(Self {
            mqtt: MqttConfig {
                broker: broker_host,
                port: broker_port,
                topic: broker_topic,
                username: mqtt_username,
                password: mqtt_password,
                client_id: mqtt_client_id,
                keep_alive_secs: mqtt_keep_alive_secs,
                clean_session: mqtt_clean_session,
                max_reconnect_attempts: mqtt_max_reconnect,
            },
            kafka: KafkaConfig {
                enabled: kafka_enabled,
                brokers: kafka_brokers,
                position_topic: kafka_position_topic,
                notifications_topic: kafka_notifications_topic,
                batch_size: kafka_batch_size,
                batch_timeout_ms: kafka_batch_timeout_ms,
                compression: kafka_compression,
                retries: kafka_retries,
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
        // Validar configuración MQTT
        if self.mqtt.broker.is_empty() {
            return Err(anyhow::anyhow!("MQTT broker no puede estar vacío"));
        }

        if self.mqtt.topic.is_empty() {
            return Err(anyhow::anyhow!("MQTT topic no puede estar vacío"));
        }

        // Validar configuración Kafka SOLO si está habilitado
        if self.kafka.enabled {
            if self.kafka.brokers.is_empty() {
                return Err(anyhow::anyhow!("Kafka brokers no puede estar vacío"));
            }

            if self.kafka.position_topic.is_empty() {
                return Err(anyhow::anyhow!("Kafka position topic no puede estar vacío"));
            }
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
                enabled: false, // Por defecto deshabilitado
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
