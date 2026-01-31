use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
use tracing::{debug, error, info};

use crate::models::{CommunicationRecord, DeviceMessage, Manufacturer};
use crate::services::DatabaseService;

#[derive(Clone)]
pub struct MessageProcessor {
    database: Arc<DatabaseService>,
    batch_size: usize,
    flush_interval: Duration,
}

impl MessageProcessor {
    pub fn new(
        database: Arc<DatabaseService>,
        batch_size: usize,
        flush_interval_ms: u64,
    ) -> Self {
        Self {
            database,
            batch_size,
            flush_interval: Duration::from_millis(flush_interval_ms),
        }
    }

    /// Inicia el procesador principal que consume mensajes del canal Kafka
    pub async fn start_processing(
        &self,
        mut message_receiver: mpsc::UnboundedReceiver<DeviceMessage>,
    ) -> Result<()> {
        info!("🚀 Iniciando procesador de mensajes...");

        // Canal interno para batch processing
        let (batch_sender, batch_receiver) = mpsc::channel::<DeviceMessage>(self.batch_size * 2);

        // Task para recibir mensajes del Kafka y enviar al batch processor
        let sender_clone = batch_sender.clone();
        tokio::spawn(async move {
            while let Some(message) = message_receiver.recv().await {
                if let Err(e) = sender_clone.send(message).await {
                    error!("Error enviando mensaje al batch processor: {}", e);
                    break;
                }
            }
            info!("Canal de recepción Kafka cerrado");
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

        // Convertir mensajes a registros de BD, agrupando por fabricante
        let mut suntech_records = Vec::new();
        let mut queclink_records = Vec::new();

        for message in batch.iter() {
            let manufacturer = message.get_manufacturer();

            // Preparar registro para BD
            match CommunicationRecord::from_device_message(message) {
                Ok(record) => {
                    // Agrupar por fabricante
                    match manufacturer {
                        Manufacturer::Suntech => suntech_records.push(record),
                        Manufacturer::Queclink => queclink_records.push(record),
                    }
                }
                Err(e) => {
                    error!(
                        "Error convirtiendo mensaje a registro de BD: {} | Device: {}, UUID: {}, Manufacturer: {:?}",
                        e, message.data.device_id, message.uuid, manufacturer
                    );
                    continue;
                }
            }
        }

        debug!(
            "📊 Agrupados: {} Suntech, {} Queclink",
            suntech_records.len(),
            queclink_records.len()
        );

        // Procesar en BD
        let db_future =
            self.process_database_batch_by_manufacturer(suntech_records, queclink_records);

        // Ejecutar operación
        let db_result = db_future.await;

        // Reportar resultados
        match db_result {
            Ok(count) => {
                debug!("✅ Guardados {} registros en BD", count);
            }
            Err(e) => {
                error!("❌ Error guardando en BD: {}", e);
            }
        }

        // Limpiar el batch
        batch.clear();
    }

    /// Procesa un lote de registros para la base de datos, agrupados por fabricante
    async fn process_database_batch_by_manufacturer(
        &self,
        suntech_records: Vec<CommunicationRecord>,
        queclink_records: Vec<CommunicationRecord>,
    ) -> Result<usize> {
        // Insertar registros directamente usando el método que separa por fabricante
        self.database
            .insert_records_by_manufacturer(suntech_records, queclink_records)
            .await
    }

    /// Procesa un lote de mensajes para Kafka

    /// Fuerza el procesamiento de todos los buffers pendientes
    pub async fn flush_all_buffers(&self) -> Result<()> {
        info!("🔄 Flushing buffer de BD...");

        if let Err(e) = self.database.flush_buffer().await {
            error!("Error haciendo flush del buffer de BD: {}", e);
        }

        Ok(())
    }

    /// Obtiene estadísticas del procesador
    pub async fn get_statistics(&self) -> ProcessorStatistics {
        let db_buffer_size = self.database.buffer_size().await;

        ProcessorStatistics {
            db_buffer_size,
            batch_size: self.batch_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessorStatistics {
    pub db_buffer_size: usize,
    pub batch_size: usize,
}
