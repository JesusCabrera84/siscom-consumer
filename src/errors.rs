use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrackingConsumerError {
    #[error("Error de configuración: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Error de base de datos: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Error de MQTT: {0}")]
    Mqtt(#[from] rumqttc::ClientError),

    #[error("Error de Kafka: {0}")]
    Kafka(#[from] rdkafka::error::KafkaError),

    #[error("Error de serialización JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Error de procesamiento: {0}")]
    Processing(String),

    #[error("Error de validación: {0}")]
    Validation(String),

    #[error("Error de conexión: {0}")]
    Connection(String),

    #[error("Error de timeout: {0}")]
    Timeout(String),

    #[error("Error interno: {0}")]
    Internal(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, TrackingConsumerError>;

impl TrackingConsumerError {
    pub fn processing(msg: impl Into<String>) -> Self {
        Self::Processing(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }

    /// Determina si el error es recuperable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Connection(_) | Self::Timeout(_) | Self::Mqtt(_) | Self::Kafka(_) => true,
            Self::Database(sqlx::Error::PoolTimedOut) => true,
            Self::Database(sqlx::Error::Io(_)) => true,
            _ => false,
        }
    }

    /// Obtiene un delay apropiado para retry basado en el tipo de error
    pub fn retry_delay(&self) -> std::time::Duration {
        match self {
            Self::Connection(_) => std::time::Duration::from_secs(5),
            Self::Timeout(_) => std::time::Duration::from_secs(2),
            Self::Mqtt(_) => std::time::Duration::from_secs(3),
            Self::Kafka(_) => std::time::Duration::from_secs(1),
            Self::Database(_) => std::time::Duration::from_secs(2),
            _ => std::time::Duration::from_secs(1),
        }
    }
}
