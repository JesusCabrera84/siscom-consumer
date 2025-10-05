use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::warn;

use super::{DeviceMessage, Manufacturer};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CommunicationRecord {
    pub id: Option<i64>,
    pub uuid: String,
    pub device_id: String,
    #[serde(skip)]
    #[sqlx(skip)]
    pub manufacturer: Option<Manufacturer>,
    pub backup_battery_voltage: Option<f64>,
    pub backup_battery_percent: Option<f64>,
    pub cell_id: Option<String>,
    pub course: Option<f64>,
    pub delivery_type: Option<String>,
    pub engine_status: Option<String>,
    pub firmware: Option<String>,
    pub fix_status: Option<String>,
    pub gps_datetime: Option<NaiveDateTime>,
    pub gps_epoch: Option<i64>,
    pub idle_time: Option<i32>,
    pub lac: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub main_battery_voltage: Option<f64>,
    pub mcc: Option<String>,
    pub mnc: Option<String>,
    pub model: Option<String>,
    pub msg_class: Option<String>,
    pub msg_counter: Option<i32>,
    pub alert_type: Option<String>,
    pub network_status: Option<String>,
    pub odometer: Option<i64>,
    pub rx_lvl: Option<i32>,
    pub satellites: Option<i32>,
    pub speed: Option<f64>,
    pub speed_time: Option<i32>,
    pub total_distance: Option<i64>,
    pub trip_distance: Option<i64>,
    pub trip_hourmeter: Option<i32>,
    pub bytes_count: Option<i32>,
    pub client_ip: Option<String>,
    pub client_port: Option<i32>,
    pub decoded_epoch: Option<i64>,
    pub received_epoch: Option<i64>,
    pub raw_message: Option<String>,
    pub received_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

impl CommunicationRecord {
    /// Convierte un DeviceMessage a un CommunicationRecord para insertar en la BD
    pub fn from_device_message(msg: &DeviceMessage) -> anyhow::Result<Self> {
        // Validación preventiva de longitudes de campos
        Self::validate_field_length("cell_id", &msg.data.cell_id, 10, &msg.data.device_id);
        Self::validate_field_length("lac", &msg.data.lac, 10, &msg.data.device_id);
        Self::validate_field_length("mcc", &msg.data.mcc, 10, &msg.data.device_id);
        Self::validate_field_length("mnc", &msg.data.mnc, 10, &msg.data.device_id);
        Self::validate_field_length("model", &msg.data.model, 50, &msg.data.device_id);
        Self::validate_field_length("firmware", &msg.data.firmware, 50, &msg.data.device_id);
        Self::validate_field_length("msg_class", &msg.data.msg_class, 20, &msg.data.device_id);

        let gps_datetime = if !msg.data.gps_datetime.is_empty() {
            chrono::NaiveDateTime::parse_from_str(&msg.data.gps_datetime, "%Y-%m-%d %H:%M:%S").ok()
        } else {
            None
        };

        let client_ip = if msg.metadata.client_ip.is_empty() {
            None
        } else {
            Some(msg.metadata.client_ip.clone())
        };

        let now = Utc::now().naive_utc();

        Ok(CommunicationRecord {
            id: None,
            uuid: msg.uuid.clone(),
            device_id: msg.data.device_id.clone(),
            manufacturer: Some(msg.get_manufacturer()),
            backup_battery_voltage: Self::parse_f64(&msg.data.backup_battery_voltage),
            backup_battery_percent: Self::parse_f64(&msg.data.backup_battery_percent),
            cell_id: Some(msg.data.cell_id.clone()),
            course: Self::parse_f64(&msg.data.course),
            delivery_type: Some(msg.data.delivery_type.clone()),
            engine_status: Some(msg.data.engine_status.clone()),
            firmware: Some(msg.data.firmware.clone()),
            fix_status: Some(msg.data.fix_status.clone()),
            gps_datetime,
            gps_epoch: Self::parse_i64(&msg.data.gps_epoch),
            idle_time: Self::parse_i32(&msg.data.idle_time),
            lac: Some(msg.data.lac.clone()),
            latitude: Self::parse_f64(&msg.data.latitude),
            longitude: Self::parse_f64(&msg.data.longitude),
            main_battery_voltage: Self::parse_f64(&msg.data.main_battery_voltage),
            mcc: Some(msg.data.mcc.clone()),
            mnc: Some(msg.data.mnc.clone()),
            model: Some(msg.data.model.clone()),
            msg_class: Some(msg.data.msg_class.clone()),
            msg_counter: Self::parse_i32(&msg.data.msg_counter),
            alert_type: if msg.data.alert.is_empty() {
                None
            } else {
                Some(msg.data.alert.clone())
            },
            network_status: Some(msg.data.network_status.clone()),
            odometer: Self::parse_i64(&msg.data.odometer),
            rx_lvl: Self::parse_i32(&msg.data.rx_lvl),
            satellites: Self::parse_i32(&msg.data.satellites),
            speed: Self::parse_f64(&msg.data.speed),
            speed_time: Self::parse_i32(&msg.data.speed_time),
            total_distance: Self::parse_i64(&msg.data.total_distance),
            trip_distance: Self::parse_i64(&msg.data.trip_distance),
            trip_hourmeter: Self::parse_i32(&msg.data.trip_hourmeter),
            bytes_count: Some(msg.metadata.bytes),
            client_ip,
            client_port: Some(msg.metadata.client_port),
            decoded_epoch: Some(msg.metadata.decoded_epoch),
            received_epoch: Some(msg.metadata.received_epoch),
            raw_message: Some(msg.raw.clone()),
            received_at: Some(now),
            created_at: Some(now),
        })
    }

    // Funciones auxiliares para parsing seguro
    fn parse_f64(s: &str) -> Option<f64> {
        if s.is_empty() {
            return None;
        }
        // Remover el signo '+' si existe
        let clean = s.strip_prefix('+').unwrap_or(s);
        clean.parse().ok()
    }

    fn parse_i64(s: &str) -> Option<i64> {
        if s.is_empty() {
            return None;
        }
        s.parse().ok()
    }

    fn parse_i32(s: &str) -> Option<i32> {
        if s.is_empty() {
            return None;
        }
        s.parse().ok()
    }

    // Validación de longitud de campos
    fn validate_field_length(field_name: &str, value: &str, max_len: usize, device_id: &str) {
        if value.len() > max_len {
            warn!(
                "⚠️ Campo '{}' excede límite en Device {}: longitud {} > {}, valor truncado: '{}'",
                field_name,
                device_id,
                value.len(),
                max_len,
                &value[..max_len.min(value.len())]
            );
        }
    }
}
