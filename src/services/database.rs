use anyhow::Result;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::models::CommunicationRecord;

#[derive(Debug, Clone)]
pub struct DatabaseService {
    pool: PgPool,
    // Buffer para batch inserts
    buffer: Arc<RwLock<Vec<CommunicationRecord>>>,
    batch_size: usize,
}

impl DatabaseService {
    pub async fn new(database_url: &str, max_connections: u32, batch_size: usize) -> Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(std::time::Duration::from_secs(600))
            .connect(database_url)
            .await?;

        // Test de conexión
        sqlx::query("SELECT 1").fetch_one(&pool).await?;

        info!("✅ Conexión a PostgreSQL establecida");

        Ok(Self {
            pool,
            buffer: Arc::new(RwLock::new(Vec::with_capacity(batch_size))),
            batch_size,
        })
    }

    /// Agrega un registro al buffer para procesamiento por lotes
    pub async fn add_to_buffer(&self, record: CommunicationRecord) -> Result<bool> {
        let mut buffer = self.buffer.write().await;
        buffer.push(record);

        // Retorna true si el buffer está lleno y necesita ser procesado
        Ok(buffer.len() >= self.batch_size)
    }

    /// Procesa todos los registros del buffer usando COPY para máximo rendimiento
    pub async fn flush_buffer(&self) -> Result<usize> {
        let mut buffer = self.buffer.write().await;
        if buffer.is_empty() {
            return Ok(0);
        }

        let count = buffer.len();
        let records = std::mem::take(&mut *buffer);
        drop(buffer); // Liberar el lock lo antes posible

        self.batch_insert(records).await?;
        Ok(count)
    }

    /// Inserción por lotes usando INSERT múltiple (simplificado)
    async fn batch_insert(&self, records: Vec<CommunicationRecord>) -> Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        self.fallback_batch_insert(&mut tx, records.clone()).await?;

        // Update current state

        self.fallback_batch_insert_current(&mut tx, &records)
            .await?;


        tx.commit().await?;
        Ok(())
    }

    /// Fallback: Inserción por lotes usando INSERT con múltiples valores
    async fn fallback_batch_insert(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        records: Vec<CommunicationRecord>,
    ) -> Result<()> {
        // Dividir en chunks más pequeños para evitar límites de PostgreSQL
        const CHUNK_SIZE: usize = 100;

        for chunk in records.chunks(CHUNK_SIZE) {
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO communications_suntech (
                    uuid, device_id, backup_battery_voltage, cell_id, course, delivery_type,
                    engine_status, firmware, fix_status, gps_datetime, gps_epoch, idle_time,
                    lac, latitude, longitude, main_battery_voltage, mcc, mnc, model,
                    msg_class, msg_counter, network_status, odometer, rx_lvl, satellites,
                    speed, speed_time, total_distance, trip_distance, trip_hourmeter,
                    bytes_count, client_ip, client_port, decoded_epoch, received_epoch,
                    raw_message, received_at, created_at
                ) ",
            );

            query_builder.push_values(chunk, |mut b, record| {
                b.push_bind(&record.uuid)
                    .push_bind(&record.device_id)
                    .push_bind(record.backup_battery_voltage)
                    .push_bind(&record.cell_id)
                    .push_bind(record.course)
                    .push_bind(&record.delivery_type)
                    .push_bind(&record.engine_status)
                    .push_bind(&record.firmware)
                    .push_bind(&record.fix_status)
                    .push_bind(record.gps_datetime)
                    .push_bind(record.gps_epoch)
                    .push_bind(record.idle_time)
                    .push_bind(&record.lac)
                    .push_bind(record.latitude)
                    .push_bind(record.longitude)
                    .push_bind(record.main_battery_voltage)
                    .push_bind(&record.mcc)
                    .push_bind(&record.mnc)
                    .push_bind(&record.model)
                    .push_bind(&record.msg_class)
                    .push_bind(record.msg_counter)
                    .push_bind(&record.network_status)
                    .push_bind(record.odometer)
                    .push_bind(record.rx_lvl)
                    .push_bind(record.satellites)
                    .push_bind(record.speed)
                    .push_bind(record.speed_time)
                    .push_bind(record.total_distance)
                    .push_bind(record.trip_distance)
                    .push_bind(record.trip_hourmeter)
                    .push_bind(record.bytes_count)
                    .push_bind(None::<String>)
                    .push_bind(record.client_port)
                    .push_bind(record.decoded_epoch)
                    .push_bind(record.received_epoch)
                    .push_bind(&record.raw_message)
                    .push_bind(record.received_at)
                    .push_bind(record.created_at);
            });

            query_builder.build().execute(&mut **tx).await?;
        }

        Ok(())
    }

    /// Fallback: Inserción por lotes usando INSERT con múltiples valores on communications_current_state
    async fn fallback_batch_insert_current(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        records: &[CommunicationRecord],
    ) -> Result<()> {
        // Dividir en chunks más pequeños para evitar límites de PostgreSQL
        const CHUNK_SIZE: usize = 100;

        for chunk in records.chunks(CHUNK_SIZE) {
            let mut query_builder = sqlx::QueryBuilder::new(
                r#"INSERT INTO communications_current_state (
                    uuid, device_id, backup_battery_voltage, cell_id, course, delivery_type,
                    engine_status, firmware, fix_status, gps_datetime, gps_epoch, idle_time,
                    lac, latitude, longitude, main_battery_voltage, mcc, mnc, model,
                    msg_class, msg_counter, network_status, odometer, rx_lvl, satellites,
                    speed, speed_time, total_distance, trip_distance, trip_hourmeter,
                    bytes_count, client_ip, client_port, decoded_epoch, received_epoch,
                    raw_message, received_at, created_at
                ) "#,
            );

            query_builder.push_values(chunk, |mut b, record| {
                b.push_bind(&record.uuid)
                    .push_bind(&record.device_id)
                    .push_bind(record.backup_battery_voltage)
                    .push_bind(&record.cell_id)
                    .push_bind(record.course)
                    .push_bind(&record.delivery_type)
                    .push_bind(&record.engine_status)
                    .push_bind(&record.firmware)
                    .push_bind(&record.fix_status)
                    .push_bind(record.gps_datetime)
                    .push_bind(record.gps_epoch)
                    .push_bind(record.idle_time)
                    .push_bind(&record.lac)
                    .push_bind(record.latitude)
                    .push_bind(record.longitude)
                    .push_bind(record.main_battery_voltage)
                    .push_bind(&record.mcc)
                    .push_bind(&record.mnc)
                    .push_bind(&record.model)
                    .push_bind(&record.msg_class)
                    .push_bind(record.msg_counter)
                    .push_bind(&record.network_status)
                    .push_bind(record.odometer)
                    .push_bind(record.rx_lvl)
                    .push_bind(record.satellites)
                    .push_bind(record.speed)
                    .push_bind(record.speed_time)
                    .push_bind(record.total_distance)
                    .push_bind(record.trip_distance)
                    .push_bind(record.trip_hourmeter)
                    .push_bind(record.bytes_count)
                    .push_bind(None::<String>)
                    .push_bind(record.client_port)
                    .push_bind(record.decoded_epoch)
                    .push_bind(record.received_epoch)
                    .push_bind(&record.raw_message)
                    .push_bind(record.received_at)
                    .push_bind(record.created_at);
            });

            query_builder.push(
                r#"
                ON CONFLICT (device_id) DO UPDATE SET
                    uuid = EXCLUDED.uuid,
                    backup_battery_voltage = EXCLUDED.backup_battery_voltage,
                    cell_id = EXCLUDED.cell_id,
                    course = EXCLUDED.course,
                    delivery_type = EXCLUDED.delivery_type,
                    engine_status = EXCLUDED.engine_status,
                    firmware = EXCLUDED.firmware,
                    fix_status = EXCLUDED.fix_status,
                    gps_datetime = EXCLUDED.gps_datetime,
                    gps_epoch = EXCLUDED.gps_epoch,
                    idle_time = EXCLUDED.idle_time,
                    lac = EXCLUDED.lac,
                    latitude = EXCLUDED.latitude,
                    longitude = EXCLUDED.longitude,
                    main_battery_voltage = EXCLUDED.main_battery_voltage,
                    mcc = EXCLUDED.mcc,
                    mnc = EXCLUDED.mnc,
                    model = EXCLUDED.model,
                    msg_class = EXCLUDED.msg_class,
                    msg_counter = EXCLUDED.msg_counter,
                    network_status = EXCLUDED.network_status,
                    odometer = EXCLUDED.odometer,
                    rx_lvl = EXCLUDED.rx_lvl,
                    satellites = EXCLUDED.satellites,
                    speed = EXCLUDED.speed,
                    speed_time = EXCLUDED.speed_time,
                    total_distance = EXCLUDED.total_distance,
                    trip_distance = EXCLUDED.trip_distance,
                    trip_hourmeter = EXCLUDED.trip_hourmeter,
                    bytes_count = EXCLUDED.bytes_count,
                    client_ip = EXCLUDED.client_ip,
                    client_port = EXCLUDED.client_port,
                    decoded_epoch = EXCLUDED.decoded_epoch,
                    received_epoch = EXCLUDED.received_epoch,
                    raw_message = EXCLUDED.raw_message,
                    received_at = NOW(),
                    created_at = EXCLUDED.created_at
                "#,
            );

            query_builder.build().execute(&mut **tx).await?;
        }

        Ok(())
    }

    /// Inserción individual para casos urgentes
    pub async fn insert_single(&self, record: CommunicationRecord) -> Result<i64> {
        let row = sqlx::query(
            r#"
            INSERT INTO communications_suntech (
                uuid, device_id, backup_battery_voltage, cell_id, course, delivery_type,
                engine_status, firmware, fix_status, gps_datetime, gps_epoch, idle_time,
                lac, latitude, longitude, main_battery_voltage, mcc, mnc, model,
                msg_class, msg_counter, network_status, odometer, rx_lvl, satellites,
                speed, speed_time, total_distance, trip_distance, trip_hourmeter,
                bytes_count, client_ip, client_port, decoded_epoch, received_epoch,
                raw_message, received_at, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30,
                $31, $32, $33, $34, $35, $36, $37, $38
            ) RETURNING id
            "#,
        )
        .bind(record.uuid)
        .bind(record.device_id)
        .bind(record.backup_battery_voltage)
        .bind(record.cell_id)
        .bind(record.course)
        .bind(record.delivery_type)
        .bind(record.engine_status)
        .bind(record.firmware)
        .bind(record.fix_status)
        .bind(record.gps_datetime)
        .bind(record.gps_epoch)
        .bind(record.idle_time)
        .bind(record.lac)
        .bind(record.latitude)
        .bind(record.longitude)
        .bind(record.main_battery_voltage)
        .bind(record.mcc)
        .bind(record.mnc)
        .bind(record.model)
        .bind(record.msg_class)
        .bind(record.msg_counter)
        .bind(record.network_status)
        .bind(record.odometer)
        .bind(record.rx_lvl)
        .bind(record.satellites)
        .bind(record.speed)
        .bind(record.speed_time)
        .bind(record.total_distance)
        .bind(record.trip_distance)
        .bind(record.trip_hourmeter)
        .bind(record.bytes_count)
        .bind(None::<String>)
        .bind(record.client_port)
        .bind(record.decoded_epoch)
        .bind(record.received_epoch)
        .bind(record.raw_message)
        .bind(record.received_at)
        .bind(record.created_at)
        .fetch_one(&self.pool)
        .await?;

        let id: i64 = row.get("id");
        Ok(id)
    }

    /// Obtiene el tamaño actual del buffer
    pub async fn buffer_size(&self) -> usize {
        self.buffer.read().await.len()
    }

    /// Verifica el estado de salud de la conexión
    pub async fn health_check(&self) -> Result<bool> {
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => Ok(true),
            Err(e) => {
                error!("Database health check failed: {}", e);
                Ok(false)
            }
        }
    }
}
