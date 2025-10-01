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
    #[error("Error interno: {0}")]
    Internal(#[from] anyhow::Error),
}
