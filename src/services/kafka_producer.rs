use anyhow::Result;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use serde_json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::models::DeviceMessage;

#[derive(Clone)]
pub struct KafkaProducerService {
    producer: FutureProducer,
    position_topic: String,
    notifications_topic: String,
    // Buffer para batch sending
    buffer: Arc<RwLock<Vec<KafkaMessage>>>,
    batch_size: usize,
}

#[derive(Debug, Clone)]
struct KafkaMessage {
    topic: String,
    key: String,
    payload: String,
}

impl KafkaProducerService {
    pub fn new(
        brokers: &[String],
        position_topic: String,
        notifications_topic: String,
        batch_size: usize,
        compression: Option<&str>,
        retries: i32,
    ) -> Result<Self> {
        let mut config = ClientConfig::new();

        config
            .set("bootstrap.servers", brokers.join(","))
            .set("message.timeout.ms", "30000")
            .set("retries", &retries.to_string())
            .set("retry.backoff.ms", "1000")
            .set("queue.buffering.max.kbytes", "32768") // 32MB buffer
            .set("linger.ms", "100"); // Agrupa mensajes por 100ms

        // Configurar compresión si está especificada
        if let Some(comp) = compression {
            config.set("compression.type", comp);
        }

        // Configurar acks para balance de velocidad/confiabilidad
        config.set("acks", "1"); // Solo esperar ack del líder

        let producer: FutureProducer = config.create()?;

        info!("✅ Kafka Producer configurado para brokers: {:?}", brokers);

        Ok(Self {
            producer,
            position_topic,
            notifications_topic,
            buffer: Arc::new(RwLock::new(Vec::with_capacity(batch_size))),
            batch_size,
        })
    }

    /// Envía un mensaje de posición a Kafka
    pub async fn send_position(&self, message: &DeviceMessage) -> Result<()> {
        let payload = serde_json::to_string(message)?;
        let key = message.data.device_id.clone();

        self.add_to_buffer(KafkaMessage {
            topic: self.position_topic.clone(),
            key,
            payload,
        })
        .await?;

        Ok(())
    }

    /// Envía una notificación a Kafka (alertas, etc.)
    pub async fn send_notification(&self, message: &DeviceMessage) -> Result<()> {
        // Solo enviar si es una alerta
        if message.data.msg_class == "ALERT" {
            let payload = serde_json::to_string(message)?;
            let key = message.data.device_id.clone();

            self.add_to_buffer(KafkaMessage {
                topic: self.notifications_topic.clone(),
                key,
                payload,
            })
            .await?;
        }

        Ok(())
    }

    /// Agrega un mensaje al buffer para envío por lotes
    async fn add_to_buffer(&self, message: KafkaMessage) -> Result<bool> {
        let mut buffer = self.buffer.write().await;
        buffer.push(message);

        // Retorna true si el buffer está lleno y necesita ser procesado
        Ok(buffer.len() >= self.batch_size)
    }

    /// Devuelve el tamaño actual del buffer de mensajes pendientes
    pub async fn buffer_size(&self) -> usize {
        self.buffer.read().await.len()
    }

    /// Procesa todos los mensajes del buffer
    pub async fn flush_buffer(&self) -> Result<usize> {
        let mut buffer = self.buffer.write().await;
        if buffer.is_empty() {
            return Ok(0);
        }

        let count = buffer.len();
        let messages = std::mem::take(&mut *buffer);
        drop(buffer); // Liberar el lock lo antes posible

        self.batch_send(messages).await?;
        Ok(count)
    }

    /// Envío por lotes para máximo rendimiento
    async fn batch_send(&self, messages: Vec<KafkaMessage>) -> Result<()> {
        if messages.is_empty() {
            return Ok(());
        }

        let mut futures = Vec::new();

        // Crear todos los records primero para evitar problemas de borrowing
        let records: Vec<_> = messages
            .iter()
            .map(|msg| {
                FutureRecord::to(&msg.topic)
                    .key(&msg.key)
                    .payload(&msg.payload)
            })
            .collect();

        for record in records {
            let future = self.producer.send(record, Duration::from_secs(30));
            futures.push(future);
        }

        // Enviar todos los mensajes en paralelo
        let results = futures::future::join_all(futures).await;

        let mut errors = 0;
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(_) => {
                    // Mensaje enviado exitosamente
                }
                Err((error, _)) => {
                    error!("Error enviando mensaje {}: {}", i, error);
                    errors += 1;
                }
            }
        }

        if errors > 0 {
            warn!(
                "Se produjeron {} errores al enviar lote de mensajes",
                errors
            );
        }

        Ok(())
    }

    /// Verifica el estado de salud del productor
    pub async fn health_check(&self) -> Result<bool> {
        // Intentar obtener metadata de los topics
        match self
            .producer
            .client()
            .fetch_metadata(None, Duration::from_secs(5))
        {
            Ok(_) => Ok(true),
            Err(e) => {
                error!("Kafka health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Cierra el productor y envía mensajes pendientes
    pub async fn shutdown(&self) -> Result<()> {
        info!("Cerrando Kafka producer...");

        // Enviar mensajes pendientes
        self.flush_buffer().await?;

        // Forzar flush del producer interno
        self.producer.flush(Duration::from_secs(30))?;

        info!("✅ Kafka producer cerrado correctamente");
        Ok(())
    }
}
