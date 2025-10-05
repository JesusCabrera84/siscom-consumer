use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
use tracing::{debug, error, info};

use crate::models::{CommunicationRecord, DeviceMessage};
use crate::services::{DatabaseService, KafkaProducerService};

#[derive(Clone)]
pub struct MessageProcessor {
    database: Arc<DatabaseService>,
    kafka: Option<Arc<KafkaProducerService>>, // Puede ser None
    batch_size: usize,
    flush_interval: Duration,
}

impl MessageProcessor {
    pub fn new(
        database: Arc<DatabaseService>,
        kafka: Option<Arc<KafkaProducerService>>,
        batch_size: usize,
        flush_interval_ms: u64,
    ) -> Self {
        Self {
            database,
            kafka,
            batch_size,
            flush_interval: Duration::from_millis(flush_interval_ms),
        }
    }

    /// Inicia el procesador principal que consume mensajes del canal MQTT
    pub async fn start_processing(
        &self,
        mut message_receiver: mpsc::UnboundedReceiver<DeviceMessage>,
    ) -> Result<()> {
        info!("🚀 Iniciando procesador de mensajes...");

        // Canal interno para batch processing
        let (batch_sender, batch_receiver) = mpsc::channel::<DeviceMessage>(self.batch_size * 2);

        // Task para recibir mensajes del MQTT y enviar al batch processor
        let sender_clone = batch_sender.clone();
        tokio::spawn(async move {
            while let Some(message) = message_receiver.recv().await {
                if let Err(e) = sender_clone.send(message).await {
                    error!("Error enviando mensaje al batch processor: {}", e);
                    break;
                }
            }
            info!("Canal de recepción MQTT cerrado");
        });

        // Task principal de procesamiento por lotes
        self.batch_processing_loop(batch_receiver).await
    }

    /// Loop principal de procesamiento por lotes
    async fn batch_processing_loop(
        &self,
        mut receiver: mpsc::Receiver<DeviceMessage>,
    ) -> Result<()> {
        let mut batch = Vec::with_capacity(self.batch_size);
        let mut flush_timer = time::interval(self.flush_interval);

        loop {
            tokio::select! {
                // Recibir mensaje
                message = receiver.recv() => {
                    match message {
                        Some(msg) => {
                            batch.push(msg);

                            // Si el batch está lleno, procesarlo inmediatamente
                            if batch.len() >= self.batch_size {
                                self.process_batch(&mut batch).await;
                            }
                        }
                        None => {
                            // Canal cerrado, procesar batch final y salir
                            if !batch.is_empty() {
                                self.process_batch(&mut batch).await;
                            }
                            break;
                        }
                    }
                }

                // Timer para flush periódico
                _ = flush_timer.tick() => {
                    if !batch.is_empty() {
                        self.process_batch(&mut batch).await;
                    }
                }
            }
        }

        info!("✅ Procesador de mensajes terminado");
        Ok(())
    }

    /// Procesa un lote de mensajes
    async fn process_batch(&self, batch: &mut Vec<DeviceMessage>) {
        if batch.is_empty() {
            return;
        }

        let batch_size = batch.len();
        debug!("📦 Procesando lote de {} mensajes", batch_size);

        // Convertir mensajes a registros de BD
        let mut db_records = Vec::with_capacity(batch_size);
        let mut kafka_messages = Vec::new();

        for message in batch.iter() {
            // Preparar registro para BD
            match CommunicationRecord::from_device_message(message) {
                Ok(record) => {
                    db_records.push(record);
                }
                Err(e) => {
                    error!(
                        "Error convirtiendo mensaje a registro de BD: {} | Device: {}, UUID: {}",
                        e, message.data.device_id, message.uuid
                    );
                    continue;
                }
            }

            // Preparar mensaje para Kafka
            kafka_messages.push(message.clone());
        }

        // Procesar en paralelo: BD + Kafka
        let db_future = self.process_database_batch(db_records);
        let kafka_future = self.process_kafka_batch_internal(kafka_messages);

        // Ejecutar ambas operaciones en paralelo
        let (db_result, kafka_result) = tokio::join!(db_future, kafka_future);

        // Reportar resultados
        match db_result {
            Ok(count) => {
                debug!("✅ Guardados {} registros en BD", count);
            }
            Err(e) => {
                error!("❌ Error guardando en BD: {}", e);
            }
        }

        match kafka_result {
            Ok(count) => {
                if self.kafka.is_some() {
                    debug!("✅ Enviados {} mensajes a Kafka", count);
                }
            }
            Err(e) => {
                if self.kafka.is_some() {
                    error!("❌ Error enviando a Kafka: {}", e);
                }
            }
        }

        // Limpiar el batch
        batch.clear();
    }

    /// Procesa un lote de registros para la base de datos
    async fn process_database_batch(&self, records: Vec<CommunicationRecord>) -> Result<usize> {
        if records.is_empty() {
            return Ok(0);
        }

        // Agregar todos los registros al buffer de la BD
        for record in records {
            if let Err(e) = self.database.add_to_buffer(record).await {
                error!("Error agregando registro al buffer de BD: {}", e);
            }
        }

        // Forzar flush del buffer
        self.database.flush_buffer().await
    }

    /// Procesa un lote de mensajes para Kafka
    async fn process_kafka_batch_internal(&self, messages: Vec<DeviceMessage>) -> Result<usize> {
        if let Some(kafka) = &self.kafka {
            if messages.is_empty() {
                return Ok(0);
            }
            let mut count = 0;
            for message in messages {
                if let Err(e) = kafka.send_position(&message).await {
                    error!("Error enviando posición a Kafka: {}", e);
                } else {
                    count += 1;
                }
                if message.data.msg_class == "ALERT" {
                    if let Err(e) = kafka.send_notification(&message).await {
                        error!("Error enviando notificación a Kafka: {}", e);
                    }
                }
            }
            if let Err(e) = kafka.flush_buffer().await {
                error!("Error haciendo flush del buffer de Kafka: {}", e);
            }
            Ok(count)
        } else {
            Ok(0)
        }
    }

    /// Fuerza el procesamiento de todos los buffers pendientes
    pub async fn flush_all_buffers(&self) -> Result<()> {
        info!("🔄 Flushing todos los buffers...");

        let db_future = self.database.flush_buffer();
        let kafka_future = async {
            if let Some(kafka) = &self.kafka {
                kafka.flush_buffer().await.ok();
            }
        };

        let (db_result, _) = tokio::join!(db_future, kafka_future);

        if let Err(e) = db_result {
            error!("Error haciendo flush del buffer de BD: {}", e);
        }

        Ok(())
    }

    /// Obtiene estadísticas del procesador
    pub async fn get_statistics(&self) -> ProcessorStatistics {
        let db_buffer_size = self.database.buffer_size().await;
        let kafka_buffer_size = if let Some(kafka) = &self.kafka {
            (**kafka).buffer_size().await
        } else {
            0
        };

        ProcessorStatistics {
            db_buffer_size,
            kafka_buffer_size,
            batch_size: self.batch_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessorStatistics {
    pub db_buffer_size: usize,
    pub kafka_buffer_size: usize,
    pub batch_size: usize,
}
