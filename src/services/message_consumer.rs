use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::models::DeviceMessage;

/// Trait para abstraer diferentes tipos de consumidores de mensajes (Kafka, etc.)
#[async_trait]
pub trait MessageConsumer: Send + Sync {
    /// Inicia el consumo de mensajes
    async fn start_consuming(&self) -> Result<UnboundedReceiver<DeviceMessage>>;

    /// Detiene el consumo de mensajes y desconecta
    async fn disconnect(&self) -> Result<()>;
}