use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::models::{CommunicationRecord, Manufacturer};

#[derive(Debug, Clone)]
pub struct DatabaseService {
    pool: PgPool,
    // Buffer para batch inserts
    buffer: Arc<RwLock<Vec<CommunicationRecord>>>,
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
        })
    }

    /// Inserta registros agrupados por fabricante
    pub async fn insert_records_by_manufacturer(
        &self,
        suntech_records: Vec<CommunicationRecord>,
        queclink_records: Vec<CommunicationRecord>,
    ) -> Result<usize> {
        let mut total = 0;

        // Insertar registros Suntech si hay
        if !suntech_records.is_empty() {
            let count = suntech_records.len();
            debug!("📦 Insertando {} registros Suntech", count);
            self.batch_insert(suntech_records, Manufacturer::Suntech)
                .await?;
            total += count;
        }

        // Insertar registros Queclink si hay
        if !queclink_records.is_empty() {
            let count = queclink_records.len();
            debug!("📦 Insertando {} registros Queclink", count);
            self.batch_insert(queclink_records, Manufacturer::Queclink)
                .await?;
            total += count;
        }

        Ok(total)
    }

    /// Procesa todos los registros del buffer agrupándolos por fabricante
    pub async fn flush_buffer(&self) -> Result<usize> {
        let mut buffer = self.buffer.write().await;
        if buffer.is_empty() {
            return Ok(0);
        }

        let count = buffer.len();
        let records = std::mem::take(&mut *buffer);
        drop(buffer); // Liberar el lock lo antes posible

        // Agrupar por fabricante
        let mut suntech_records = Vec::new();
        let mut queclink_records = Vec::new();

        for record in records {
            match record.manufacturer {
                Some(Manufacturer::Suntech) => suntech_records.push(record),
                Some(Manufacturer::Queclink) => queclink_records.push(record),
                None => {
                    warn!("Registro sin fabricante asignado, usando Suntech por defecto");
                    suntech_records.push(record);
                }
            }
        }

        // Insertar usando el método que agrupa por fabricante
        self.insert_records_by_manufacturer(suntech_records, queclink_records)
            .await?;
        Ok(count)
    }

    /// Inserción por lotes usando INSERT múltiple (simplificado)
    async fn batch_insert(
        &self,
        records: Vec<CommunicationRecord>,
        manufacturer: Manufacturer,
    ) -> Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        let table_name = match manufacturer {
            Manufacturer::Suntech => "communications_suntech",
            Manufacturer::Queclink => "communications_queclink",
        };

        let mut tx = self.pool.begin().await?;

        self.fallback_batch_insert(&mut tx, records.clone(), table_name)
            .await?;

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
        table_name: &str,
    ) -> Result<()> {
        // Dividir en chunks más pequeños para evitar límites de PostgreSQL
        const CHUNK_SIZE: usize = 100;

        for chunk in records.chunks(CHUNK_SIZE) {
            let query = format!(
                "INSERT INTO {} (
                    uuid, device_id, backup_battery_voltage, backup_battery_percent, cell_id, course, delivery_type,
                    engine_status, firmware, fix_status, gps_datetime, gps_epoch, idle_time,
                    lac, latitude, longitude, main_battery_voltage, mcc, mnc, model,
                    msg_class, msg_counter, alert_type, network_status, odometer, rx_lvl, satellites,
                    speed, speed_time, total_distance, trip_distance, trip_hourmeter,
                    bytes_count, client_ip, client_port, decoded_epoch, received_epoch,
                    raw_message, received_at, created_at
                ) ",
                table_name
            );
            let mut query_builder = sqlx::QueryBuilder::new(query);

            query_builder.push_values(chunk, |mut b, record| {
                b.push_bind(&record.uuid)
                    .push_bind(&record.device_id)
                    .push_bind(record.backup_battery_voltage)
                    .push_bind(record.backup_battery_percent)
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
                    .push_bind(&record.alert_type)
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
                    .push_bind(&record.client_ip)
                    .push_bind(record.client_port)
                    .push_bind(record.decoded_epoch)
                    .push_bind(record.received_epoch)
                    .push_bind(&record.raw_message)
                    .push_bind(record.received_at)
                    .push_bind(record.created_at);
            });

            match query_builder.build().execute(&mut **tx).await {
                Ok(_) => {}
                Err(e) => {
                    error!("❌ Error insertando batch en {}: {}", table_name, e);
                    // Log de los registros problemáticos
                    for (idx, record) in chunk.iter().enumerate() {
                        warn!(
                            "📝 Registro #{} - Device: {}, UUID: {}, Cell ID len: {}, LAC len: {}, MCC len: {}, MNC len: {}",
                            idx,
                            record.device_id,
                            record.uuid,
                            record.cell_id.as_ref().map(|s| s.len()).unwrap_or(0),
                            record.lac.as_ref().map(|s| s.len()).unwrap_or(0),
                            record.mcc.as_ref().map(|s| s.len()).unwrap_or(0),
                            record.mnc.as_ref().map(|s| s.len()).unwrap_or(0),
                        );
                        // Log campos que comúnmente tienen límites VARCHAR(10)
                        Self::log_field_if_too_long("cell_id", record.cell_id.as_deref(), 10);
                        Self::log_field_if_too_long("lac", record.lac.as_deref(), 10);
                        Self::log_field_if_too_long("mcc", record.mcc.as_deref(), 10);
                        Self::log_field_if_too_long("mnc", record.mnc.as_deref(), 10);
                        Self::log_field_if_too_long("model", record.model.as_deref(), 50);
                        Self::log_field_if_too_long("firmware", record.firmware.as_deref(), 50);
                        Self::log_field_if_too_long("msg_class", record.msg_class.as_deref(), 20);
                    }
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }

    /// Helper para loguear campos que exceden el límite
    fn log_field_if_too_long(field_name: &str, value: Option<&str>, max_len: usize) {
        if let Some(val) = value {
            if val.len() > max_len {
                error!(
                    "🚨 Campo '{}' excede límite: longitud {} > {}, valor: '{}'",
                    field_name,
                    val.len(),
                    max_len,
                    val
                );
            }
        }
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
                    uuid, device_id, backup_battery_voltage, backup_battery_percent, cell_id, course, delivery_type,
                    engine_status, firmware, fix_status, gps_datetime, gps_epoch, idle_time,
                    lac, latitude, longitude, main_battery_voltage, mcc, mnc, model,
                    msg_class, msg_counter, alert_type, network_status, odometer, rx_lvl, satellites,
                    speed, speed_time, total_distance, trip_distance, trip_hourmeter,
                    bytes_count, client_ip, client_port, decoded_epoch, received_epoch,
                    raw_message, received_at, created_at
                ) "#,
            );

            query_builder.push_values(chunk, |mut b, record| {
                b.push_bind(&record.uuid)
                    .push_bind(&record.device_id)
                    .push_bind(record.backup_battery_voltage)
                    .push_bind(record.backup_battery_percent)
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
                    .push_bind(&record.alert_type)
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
                    .push_bind(&record.client_ip)
                    .push_bind(record.client_port)
                    .push_bind(record.decoded_epoch)
                    .push_bind(record.received_epoch)
                    .push_bind(&record.raw_message)
                    .push_bind(record.received_at)
                    .push_bind(record.created_at);
            });

            query_builder.push(
                r#"
                ON CONFLICT (device_id, msg_class) DO UPDATE SET
                    uuid = EXCLUDED.uuid,
                    backup_battery_voltage = EXCLUDED.backup_battery_voltage,
                    backup_battery_percent = EXCLUDED.backup_battery_percent,
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
                    alert_type = EXCLUDED.alert_type,
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

            match query_builder.build().execute(&mut **tx).await {
                Ok(_) => {}
                Err(e) => {
                    error!(
                        "❌ Error insertando batch en communications_current_state: {}",
                        e
                    );
                    // Log de los registros problemáticos
                    for (idx, record) in chunk.iter().enumerate() {
                        warn!(
                            "📝 Registro #{} - Device: {}, UUID: {}, Cell ID len: {}, LAC len: {}, MCC len: {}, MNC len: {}",
                            idx,
                            record.device_id,
                            record.uuid,
                            record.cell_id.as_ref().map(|s| s.len()).unwrap_or(0),
                            record.lac.as_ref().map(|s| s.len()).unwrap_or(0),
                            record.mcc.as_ref().map(|s| s.len()).unwrap_or(0),
                            record.mnc.as_ref().map(|s| s.len()).unwrap_or(0),
                        );
                        // Log campos que comúnmente tienen límites VARCHAR(10)
                        Self::log_field_if_too_long("cell_id", record.cell_id.as_deref(), 10);
                        Self::log_field_if_too_long("lac", record.lac.as_deref(), 10);
                        Self::log_field_if_too_long("mcc", record.mcc.as_deref(), 10);
                        Self::log_field_if_too_long("mnc", record.mnc.as_deref(), 10);
                        Self::log_field_if_too_long("model", record.model.as_deref(), 50);
                        Self::log_field_if_too_long("firmware", record.firmware.as_deref(), 50);
                        Self::log_field_if_too_long("msg_class", record.msg_class.as_deref(), 20);
                    }
                    return Err(e.into());
                }
            }
        }

        Ok(())
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
