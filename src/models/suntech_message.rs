use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuntechMessage {
    pub data: SuntechData,
    pub decoded: SuntechDecoded,
    pub metadata: SuntechMetadata,
    pub raw: String,
    pub uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuntechData {
    #[serde(rename = "BACKUP_BATTERY_VOLTAGE")]
    pub backup_battery_voltage: String,
    #[serde(rename = "CELL_ID")]
    pub cell_id: String,
    #[serde(rename = "COURSE")]
    pub course: String,
    #[serde(rename = "DELIVERY_TYPE")]
    pub delivery_type: String,
    #[serde(rename = "DEVICE_ID")]
    pub device_id: String,
    #[serde(rename = "ENGINE_STATUS")]
    pub engine_status: String,
    #[serde(rename = "FIRMWARE")]
    pub firmware: String,
    #[serde(rename = "FIX_")]
    pub fix_status: String,
    #[serde(rename = "GPS_DATETIME")]
    pub gps_datetime: String,
    #[serde(rename = "GPS_EPOCH")]
    pub gps_epoch: String,
    #[serde(rename = "IDLE_TIME")]
    pub idle_time: String,
    #[serde(rename = "LAC")]
    pub lac: String,
    #[serde(rename = "LATITUD")]
    pub latitude: String,
    #[serde(rename = "LONGITUD")]
    pub longitude: String,
    #[serde(rename = "MAIN_BATTERY_VOLTAGE")]
    pub main_battery_voltage: String,
    #[serde(rename = "MCC")]
    pub mcc: String,
    #[serde(rename = "MNC")]
    pub mnc: String,
    #[serde(rename = "MODEL")]
    pub model: String,
    #[serde(rename = "MSG_CLASS")]
    pub msg_class: String,
    #[serde(rename = "MSG_COUNTER")]
    pub msg_counter: String,
    #[serde(rename = "NETWORK_STATUS")]
    pub network_status: String,
    #[serde(rename = "ODOMETER")]
    pub odometer: String,
    #[serde(rename = "RX_LVL")]
    pub rx_lvl: String,
    #[serde(rename = "SATELLITES")]
    pub satellites: String,
    #[serde(rename = "SPEED")]
    pub speed: String,
    #[serde(rename = "SPEED_TIME")]
    pub speed_time: String,
    #[serde(rename = "TOTAL_DISTANCE")]
    pub total_distance: String,
    #[serde(rename = "TRIP_DISTANCE")]
    pub trip_distance: String,
    #[serde(rename = "TRIP_HOURMETER")]
    pub trip_hourmeter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuntechDecoded {
    #[serde(rename = "SuntechRaw")]
    pub suntech_raw: SuntechRaw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuntechRaw {
    #[serde(rename = "ASSIGN_MAP")]
    pub assign_map: String,
    #[serde(rename = "AXIST_Y")]
    pub axis_y: String,
    #[serde(rename = "AXIS_X")]
    pub axis_x: String,
    #[serde(rename = "AXIS_Z")]
    pub axis_z: String,
    #[serde(rename = "CELL_ID")]
    pub cell_id: String,
    #[serde(rename = "CRS")]
    pub course: String,
    #[serde(rename = "DEVICE_ID")]
    pub device_id: String,
    #[serde(rename = "FIX")]
    pub fix: String,
    #[serde(rename = "FW")]
    pub firmware: String,
    #[serde(rename = "GPS_DATE")]
    pub gps_date: String,
    #[serde(rename = "GPS_TIME")]
    pub gps_time: String,
    #[serde(rename = "HEADER")]
    pub header: String,
    #[serde(rename = "IDLE_TIME")]
    pub idle_time: String,
    #[serde(rename = "IN_STATE")]
    pub in_state: String,
    #[serde(rename = "LAC")]
    pub lac: String,
    #[serde(rename = "LAT")]
    pub latitude: String,
    #[serde(rename = "LON")]
    pub longitude: String,
    #[serde(rename = "MCC")]
    pub mcc: String,
    #[serde(rename = "MNC")]
    pub mnc: String,
    #[serde(rename = "MODEL")]
    pub model: String,
    #[serde(rename = "MODE_MAP")]
    pub mode_map: String,
    #[serde(rename = "MSG_NUM")]
    pub msg_num: String,
    #[serde(rename = "MSG_TYPE")]
    pub msg_type: String,
    #[serde(rename = "NET_STATUS")]
    pub net_status: String,
    #[serde(rename = "ODOMETER_MTS")]
    pub odometer_mts: String,
    #[serde(rename = "OUT_STATE")]
    pub out_state: String,
    #[serde(rename = "REPORT_MAP")]
    pub report_map: String,
    #[serde(rename = "RX_LVL")]
    pub rx_lvl: String,
    #[serde(rename = "SAT")]
    pub satellites: String,
    #[serde(rename = "SPD")]
    pub speed: String,
    #[serde(rename = "SPEED_TIME")]
    pub speed_time: String,
    #[serde(rename = "STT_RPT_TYPE")]
    pub stt_rpt_type: String,
    #[serde(rename = "TOTAL_DISTANCE")]
    pub total_distance: String,
    #[serde(rename = "TRIP_DISTANCE")]
    pub trip_distance: String,
    #[serde(rename = "TRIP_HOURMETER")]
    pub trip_hourmeter: String,
    #[serde(rename = "VOLT_BACKUP")]
    pub volt_backup: String,
    #[serde(rename = "VOLT_MAIN")]
    pub volt_main: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuntechMetadata {
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
}
