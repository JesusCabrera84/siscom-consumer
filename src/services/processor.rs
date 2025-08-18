use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
use tracing::{debug, error, info};

use crate::models::{CommunicationRecord, SuntechMessage};
use crate::services::{DatabaseService, KafkaProducerService};

#[derive(Clone)]
pub struct MessageProcessor {
    database: Arc<DatabaseService>,
    kafka: Arc<KafkaProducerService>,
    batch_size: usize,
    flush_interval: Duration,
}

impl MessageProcessor {
    pub fn new(
        database: Arc<DatabaseService>,
        kafka: Arc<KafkaProducerService>,
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
        mut message_receiver: mpsc::UnboundedReceiver<SuntechMessage>,
    ) -> Result<()> {
        info!("🚀 Iniciando procesador de mensajes...");

        // Canal interno para batch processing
        let (batch_sender, batch_receiver) = mpsc::channel::<SuntechMessage>(self.batch_size * 2);

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
        mut receiver: mpsc::Receiver<SuntechMessage>,
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
    async fn process_batch(&self, batch: &mut Vec<SuntechMessage>) {
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
            match CommunicationRecord::from_suntech_message(message) {
                Ok(record) => {
                    db_records.push(record);
                }
                Err(e) => {
                    error!("Error convirtiendo mensaje a registro de BD: {}", e);
                    continue;
                }
            }

            // Preparar mensaje para Kafka
            kafka_messages.push(message.clone());
        }

        // Procesar en paralelo: BD + Kafka
        let db_future = self.process_database_batch(db_records);
        let kafka_future = self.process_kafka_batch(kafka_messages);

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
                debug!("✅ Enviados {} mensajes a Kafka", count);
            }
            Err(e) => {
                error!("❌ Error enviando a Kafka: {}", e);
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
    async fn process_kafka_batch(&self, messages: Vec<SuntechMessage>) -> Result<usize> {
        if messages.is_empty() {
            return Ok(0);
        }

        let mut count = 0;

        // Agregar mensajes a los buffers de Kafka
        for message in messages {
            // Enviar a topic de posiciones
            if let Err(e) = self.kafka.send_position(&message).await {
                error!("Error enviando posición a Kafka: {}", e);
            } else {
                count += 1;
            }

            // Enviar a topic de notificaciones si es alerta
            if message.data.msg_class == "ALERT" {
                if let Err(e) = self.kafka.send_notification(&message).await {
                    error!("Error enviando notificación a Kafka: {}", e);
                }
            }
        }

        // Forzar flush del buffer de Kafka
        if let Err(e) = self.kafka.flush_buffer().await {
            error!("Error haciendo flush del buffer de Kafka: {}", e);
        }

        Ok(count)
    }

    /// Procesa un mensaje individual (para casos urgentes)
    pub async fn process_single_message(&self, message: SuntechMessage) -> Result<()> {
        debug!("🚨 Procesando mensaje individual para dispositivo: {}", message.data.device_id);

        // Convertir a registro de BD
        let record = CommunicationRecord::from_suntech_message(&message)?;

        // Procesar en paralelo
        let db_future = self.database.insert_single(record);
        let kafka_position_future = self.kafka.send_position(&message);
        let kafka_notification_future = async {
            if message.data.msg_class == "ALERT" {
                self.kafka.send_notification(&message).await
            } else {
                Ok(())
            }
        };

        let (db_result, kafka_pos_result, kafka_notif_result) = 
            tokio::join!(db_future, kafka_position_future, kafka_notification_future);

        // Verificar resultados
        match db_result {
            Err(e) => error!("Error guardando mensaje individual en BD: {}", e),
            _ => {}
        }

        match kafka_pos_result {
            Err(e) => error!("Error enviando posición individual a Kafka: {}", e),
            _ => {}
        }

        match kafka_notif_result {
            Err(e) => error!("Error enviando notificación individual a Kafka: {}", e),
            _ => {}
        }

        Ok(())
    }

    /// Fuerza el procesamiento de todos los buffers pendientes
    pub async fn flush_all_buffers(&self) -> Result<()> {
        info!("🔄 Flushing todos los buffers...");

        let db_future = self.database.flush_buffer();
        let kafka_future = self.kafka.flush_buffer();

        let (db_result, kafka_result) = tokio::join!(db_future, kafka_future);

        if let Err(e) = db_result {
            error!("Error haciendo flush del buffer de BD: {}", e);
        }

        if let Err(e) = kafka_result {
            error!("Error haciendo flush del buffer de Kafka: {}", e);
        }

        Ok(())
    }

    /// Obtiene estadísticas del procesador
    pub async fn get_statistics(&self) -> ProcessorStatistics {
        let db_buffer_size = self.database.buffer_size().await;
        let kafka_buffer_size = self.kafka.buffer_size().await;

        ProcessorStatistics {
            db_buffer_size,
            kafka_buffer_size,
            batch_size: self.batch_size,
            flush_interval_ms: self.flush_interval.as_millis() as u64,
        }
    }

    /// Verifica el estado de salud del procesador
    pub async fn health_check(&self) -> ProcessorHealth {
        let db_healthy = self.database.health_check().await.unwrap_or(false);
        let kafka_healthy = self.kafka.health_check().await.unwrap_or(false);

        ProcessorHealth {
            database_healthy: db_healthy,
            kafka_healthy,
            overall_healthy: db_healthy && kafka_healthy,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessorStatistics {
    pub db_buffer_size: usize,
    pub kafka_buffer_size: usize,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessorHealth {
    pub database_healthy: bool,
    pub kafka_healthy: bool,
    pub overall_healthy: bool,
}
