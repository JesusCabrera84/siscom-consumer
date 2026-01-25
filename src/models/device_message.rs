use serde::{Deserialize, Serialize};

/// Enum que representa los fabricantes de dispositivos soportados
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Manufacturer {
    Suntech,
    Queclink,
}

/// Estructura principal que representa un mensaje de dispositivo estandarizado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceMessage {
    pub data: DeviceData,
    pub decoded: DecodedData,
    pub metadata: DeviceMetadata,
    pub raw: String,
    pub uuid: String,
}

impl DeviceMessage {
    /// Determina el fabricante del dispositivo basándose en el contenido del campo decoded
    pub fn get_manufacturer(&self) -> Manufacturer {
        match &self.decoded {
            DecodedData::Suntech { .. } => Manufacturer::Suntech,
            DecodedData::Queclink { .. } => Manufacturer::Queclink,
        }
    }
}

/// Datos estandarizados del dispositivo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceData {
    #[serde(rename = "ALERT", default)]
    pub alert: String,
    #[serde(rename = "ALTITUDE", default)]
    pub altitude: String,
    #[serde(rename = "BACKUP_BATTERY_VOLTAGE", default)]
    pub backup_battery_voltage: String,
    #[serde(rename = "PERCENT_BACKUP", default)]
    pub backup_battery_percent: String,
    #[serde(rename = "CELL_ID", default)]
    pub cell_id: String,
    #[serde(rename = "COURSE", default)]
    pub course: String,
    #[serde(rename = "DELIVERY_TYPE", default)]
    pub delivery_type: String,
    #[serde(rename = "DEVICE_ID")]
    pub device_id: String,
    #[serde(rename = "ENGINE_STATUS", default)]
    pub engine_status: String,
    #[serde(rename = "FIRMWARE", default)]
    pub firmware: String,
    #[serde(rename = "FIX_", default)]
    pub fix_status: String,
    #[serde(rename = "GPS_DATETIME", default)]
    pub gps_datetime: String,
    #[serde(rename = "GPS_EPOCH", default)]
    pub gps_epoch: String,
    #[serde(rename = "IDLE_TIME", default)]
    pub idle_time: String,
    #[serde(rename = "LAC", default)]
    pub lac: String,
    #[serde(rename = "LATITUD", default)]
    pub latitude: String,
    #[serde(rename = "LONGITUD", default)]
    pub longitude: String,
    #[serde(rename = "MAIN_BATTERY_VOLTAGE", default)]
    pub main_battery_voltage: String,
    #[serde(rename = "MCC", default)]
    pub mcc: String,
    #[serde(rename = "MNC", default)]
    pub mnc: String,
    #[serde(rename = "MODEL", default)]
    pub model: String,
    #[serde(rename = "MSG_CLASS", default)]
    pub msg_class: String,
    #[serde(rename = "MSG_COUNTER", default)]
    pub msg_counter: String,
    #[serde(rename = "NETWORK_STATUS", default)]
    pub network_status: String,
    #[serde(rename = "ODOMETER", default)]
    pub odometer: String,
    #[serde(rename = "RX_LVL", default)]
    pub rx_lvl: String,
    #[serde(rename = "SATELLITES", default)]
    pub satellites: String,
    #[serde(rename = "SPEED", default)]
    pub speed: String,
    #[serde(rename = "SPEED_TIME", default)]
    pub speed_time: String,
    #[serde(rename = "TOTAL_DISTANCE", default)]
    pub total_distance: String,
    #[serde(rename = "TRIP_DISTANCE", default)]
    pub trip_distance: String,
    #[serde(rename = "TRIP_HOURMETER", default)]
    pub trip_hourmeter: String,
}

/// Enum que soporta diferentes formatos de datos decodificados según el fabricante
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DecodedData {
    Suntech {
        #[serde(rename = "SuntechRaw")]
        suntech_raw: SuntechRaw,
    },
    Queclink {
        #[serde(rename = "QueclinkRaw")]
        queclink_raw: QueclinkRaw,
    },
}

/// Datos raw de dispositivos Queclink
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueclinkRaw {
    #[serde(rename = "ALTITUDE", default)]
    pub altitude: String,
    #[serde(rename = "CELL_ID", default)]
    pub cell_id: String,
    #[serde(rename = "CRS", default)]
    pub course: String,
    #[serde(rename = "DEVICE_ID", default)]
    pub device_id: String,
    #[serde(rename = "FIX", default)]
    pub fix: String,
    #[serde(rename = "GPS_DATE_TIME", default)]
    pub gps_date_time: String,
    #[serde(rename = "HEADER", default)]
    pub header: String,
    #[serde(rename = "LAC", default)]
    pub lac: String,
    #[serde(rename = "LAT", default)]
    pub latitude: String,
    #[serde(rename = "LON", default)]
    pub longitude: String,
    #[serde(rename = "MCC", default)]
    pub mcc: String,
    #[serde(rename = "MNC", default)]
    pub mnc: String,
    #[serde(rename = "MSG_NUM", default)]
    pub msg_num: String,
    #[serde(rename = "PROTOCOL_VERSION", default)]
    pub protocol_version: String,
    #[serde(rename = "RESERVED", default)]
    pub reserved: String,
    #[serde(rename = "SEND_DATE_TIME", default)]
    pub send_date_time: String,
    #[serde(rename = "SPD", default)]
    pub speed: String,
}

/// Datos raw de dispositivos Suntech
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SuntechRaw {
    #[serde(rename = "ASSIGN_MAP", default)]
    pub assign_map: String,
    #[serde(rename = "AXIST_Y", default)]
    pub axis_y: String,
    #[serde(rename = "AXIS_X", default)]
    pub axis_x: String,
    #[serde(rename = "AXIS_Z", default)]
    pub axis_z: String,
    #[serde(rename = "CELL_ID", default)]
    pub cell_id: String,
    #[serde(rename = "CRS", default)]
    pub course: String,
    #[serde(rename = "DEVICE_ID", default)]
    pub device_id: String,
    #[serde(rename = "FIX", default)]
    pub fix: String,
    #[serde(rename = "FW", default)]
    pub firmware: String,
    #[serde(rename = "GPS_DATE", default)]
    pub gps_date: String,
    #[serde(rename = "GPS_TIME", default)]
    pub gps_time: String,
    #[serde(rename = "HEADER", default)]
    pub header: String,
    #[serde(rename = "IDLE_TIME", default)]
    pub idle_time: String,
    #[serde(rename = "IN_STATE", default)]
    pub in_state: String,
    #[serde(rename = "LAC", default)]
    pub lac: String,
    #[serde(rename = "LAT", default)]
    pub latitude: String,
    #[serde(rename = "LON", default)]
    pub longitude: String,
    #[serde(rename = "MCC", default)]
    pub mcc: String,
    #[serde(rename = "MNC", default)]
    pub mnc: String,
    #[serde(rename = "MODEL", default)]
    pub model: String,
    #[serde(rename = "MODE_MAP", default)]
    pub mode_map: String,
    #[serde(rename = "MSG_NUM", default)]
    pub msg_num: String,
    #[serde(rename = "MSG_TYPE", default)]
    pub msg_type: String,
    #[serde(rename = "NET_STATUS", default)]
    pub net_status: String,
    #[serde(rename = "ODOMETER_MTS", default)]
    pub odometer_mts: String,
    #[serde(rename = "OUT_STATE", default)]
    pub out_state: String,
    #[serde(rename = "REPORT_MAP", default)]
    pub report_map: String,
    #[serde(rename = "RX_LVL", default)]
    pub rx_lvl: String,
    #[serde(rename = "SAT", default)]
    pub satellites: String,
    #[serde(rename = "SPD", default)]
    pub speed: String,
    #[serde(rename = "SPEED_TIME", default)]
    pub speed_time: String,
    #[serde(rename = "STT_RPT_TYPE", default)]
    pub stt_rpt_type: String,
    #[serde(rename = "TOTAL_DISTANCE", default)]
    pub total_distance: String,
    #[serde(rename = "TRIP_DISTANCE", default)]
    pub trip_distance: String,
    #[serde(rename = "TRIP_HOURMETER", default)]
    pub trip_hourmeter: String,
    #[serde(rename = "VOLT_BACKUP", default)]
    pub volt_backup: String,
    #[serde(rename = "VOLT_MAIN", default)]
    pub volt_main: String,
}

/// Metadatos del mensaje (información del servidor receptor)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceMetadata {
    #[serde(rename = "BYTES")]
    pub bytes: i32,
    #[serde(rename = "CLIENT_IP")]
    pub client_ip: String,
    #[serde(rename = "CLIENT_PORT")]
    pub client_port: i32,
    #[serde(rename = "DECODED_EPOCH")]
    pub decoded_epoch: i64,
    #[serde(rename = "RECEIVED_EPOCH")]
    pub received_epoch: i64,
    #[serde(rename = "WORKER_ID", default)]
    pub worker_id: i32,
}
