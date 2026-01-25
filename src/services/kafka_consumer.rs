use anyhow::Result;
use async_trait::async_trait;
use prost::Message as ProstMessage;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::Message;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::config::BrokerConfig;
use crate::models::DeviceMessage;
use crate::services::MessageConsumer;

/// Servicio consumidor de Kafka que lee mensajes protobuf
#[derive(Clone)]
pub struct KafkaConsumerService {
    consumer: Arc<StreamConsumer>,
    topic: String,
}

impl KafkaConsumerService {
    /// Crea un nuevo consumidor Kafka
    pub fn new(config: &BrokerConfig) -> Result<Self> {
        // Crear configuración base con binding para evitar problemas de lifetime
        let mut binding = ClientConfig::new();
        let base_config = binding
            .set("bootstrap.servers", &config.host)
            .set("group.id", "siscom-consumer-group")
            .set("auto.offset.reset", "latest")
            .set("enable.auto.commit", "true")
            .set("auto.commit.interval.ms", "1000")
            .set("session.timeout.ms", "6000");

        // Configurar SASL authentication si las variables de entorno están presentes
        let client_config = if let Ok(security_protocol) = std::env::var("KAFKA_SECURITY_PROTOCOL") {
            info!("🔐 Configurando security.protocol: {}", security_protocol);
            base_config.set("security.protocol", security_protocol)
        } else {
            base_config
        };

        let client_config = if let Ok(sasl_mechanism) = std::env::var("KAFKA_SASL_MECHANISM") {
            info!("🔐 Configurando sasl.mechanism: {}", sasl_mechanism);
            client_config.set("sasl.mechanism", sasl_mechanism)
        } else {
            client_config
        };

        let client_config = if let Ok(username) = std::env::var("KAFKA_USERNAME") {
            info!("🔐 Configurando sasl.username: {}", username);
            client_config.set("sasl.username", username)
        } else {
            client_config
        };

        let client_config = if let Ok(password) = std::env::var("KAFKA_PASSWORD") {
            info!("🔐 Configurando sasl.password: [PROTECTED]");
            client_config.set("sasl.password", password)
        } else {
            client_config
        };

        let consumer: StreamConsumer = client_config.create()?;

        info!("✅ Kafka Consumer configurado para broker: {}", config.host);

        Ok(Self {
            consumer: Arc::new(consumer),
            topic: config.topic.clone(),
        })
    }

    /// Convierte un mensaje protobuf KafkaMessage a DeviceMessage
    fn kafka_message_to_device_message(
        kafka_msg: &crate::config::siscom::KafkaMessage,
    ) -> Result<DeviceMessage> {
        // Extraer datos normalizados del mapa
        let data_map = &kafka_msg.data;
        let metadata = kafka_msg.metadata.as_ref().ok_or_else(|| anyhow::anyhow!("Missing metadata in KafkaMessage"))?;

        // Crear DeviceMessage desde los datos protobuf
        let device_message = DeviceMessage {
            data: crate::models::DeviceData {
                alert: data_map.get("ALERT").cloned().unwrap_or_default(),
                altitude: data_map.get("ALTITUDE").cloned().unwrap_or_default(),
                backup_battery_voltage: data_map.get("BACKUP_BATTERY_VOLTAGE").cloned().unwrap_or_default(),
                backup_battery_percent: data_map.get("PERCENT_BACKUP").cloned().unwrap_or_default(),
                cell_id: data_map.get("CELL_ID").cloned().unwrap_or_default(),
                course: data_map.get("COURSE").cloned().unwrap_or_default(),
                delivery_type: data_map.get("DELIVERY_TYPE").cloned().unwrap_or_default(),
                device_id: data_map.get("DEVICE_ID").cloned().unwrap_or_default(),
                engine_status: data_map.get("ENGINE_STATUS").cloned().unwrap_or_default(),
                firmware: data_map.get("FIRMWARE").cloned().unwrap_or_default(),
                fix_status: data_map.get("FIX_").cloned().unwrap_or_default(),
                gps_datetime: data_map.get("GPS_DATETIME").cloned().unwrap_or_default(),
                gps_epoch: data_map.get("GPS_EPOCH").cloned().unwrap_or_default(),
                idle_time: data_map.get("IDLE_TIME").cloned().unwrap_or_default(),
                lac: data_map.get("LAC").cloned().unwrap_or_default(),
                latitude: data_map.get("LATITUD").cloned().unwrap_or_default(),
                longitude: data_map.get("LONGITUD").cloned().unwrap_or_default(),
                main_battery_voltage: data_map.get("MAIN_BATTERY_VOLTAGE").cloned().unwrap_or_default(),
                mcc: data_map.get("MCC").cloned().unwrap_or_default(),
                mnc: data_map.get("MNC").cloned().unwrap_or_default(),
                model: data_map.get("MODEL").cloned().unwrap_or_default(),
                msg_class: data_map.get("MSG_CLASS").cloned().unwrap_or_default(),
                msg_counter: data_map.get("MSG_COUNTER").cloned().unwrap_or_default(),
                network_status: data_map.get("NETWORK_STATUS").cloned().unwrap_or_default(),
                odometer: data_map.get("ODOMETER").cloned().unwrap_or_default(),
                rx_lvl: data_map.get("RX_LVL").cloned().unwrap_or_default(),
                satellites: data_map.get("SATELLITES").cloned().unwrap_or_default(),
                speed: data_map.get("SPEED").cloned().unwrap_or_default(),
                speed_time: data_map.get("SPEED_TIME").cloned().unwrap_or_default(),
                total_distance: data_map.get("TOTAL_DISTANCE").cloned().unwrap_or_default(),
                trip_distance: data_map.get("TRIP_DISTANCE").cloned().unwrap_or_default(),
                trip_hourmeter: data_map.get("TRIP_HOURMETER").cloned().unwrap_or_default(),
            },
            decoded: match &kafka_msg.decoded {
                Some(crate::config::siscom::kafka_message::Decoded::Suntech(suntech)) => {
                    crate::models::DecodedData::Suntech {
                        suntech_raw: crate::models::SuntechRaw {
                            assign_map: suntech.fields.get("ASSIGN_MAP").cloned().unwrap_or_default(),
                            axis_x: suntech.fields.get("AXIS_X").cloned().unwrap_or_default(),
                            axis_y: suntech.fields.get("AXIST_Y").cloned().unwrap_or_default(),
                            axis_z: suntech.fields.get("AXIS_Z").cloned().unwrap_or_default(),
                            cell_id: suntech.fields.get("CELL_ID").cloned().unwrap_or_default(),
                            course: suntech.fields.get("CRS").cloned().unwrap_or_default(),
                            device_id: suntech.fields.get("DEVICE_ID").cloned().unwrap_or_default(),
                            fix: suntech.fields.get("FIX").cloned().unwrap_or_default(),
                            firmware: suntech.fields.get("FW").cloned().unwrap_or_default(),
                            gps_date: suntech.fields.get("GPS_DATE").cloned().unwrap_or_default(),
                            gps_time: suntech.fields.get("GPS_TIME").cloned().unwrap_or_default(),
                            header: suntech.fields.get("HEADER").cloned().unwrap_or_default(),
                            idle_time: suntech.fields.get("IDLE_TIME").cloned().unwrap_or_default(),
                            in_state: suntech.fields.get("IN_STATE").cloned().unwrap_or_default(),
                            lac: suntech.fields.get("LAC").cloned().unwrap_or_default(),
                            latitude: suntech.fields.get("LAT").cloned().unwrap_or_default(),
                            longitude: suntech.fields.get("LON").cloned().unwrap_or_default(),
                            mcc: suntech.fields.get("MCC").cloned().unwrap_or_default(),
                            mnc: suntech.fields.get("MNC").cloned().unwrap_or_default(),
                            model: suntech.fields.get("MODEL").cloned().unwrap_or_default(),
                            mode_map: suntech.fields.get("MODE_MAP").cloned().unwrap_or_default(),
                            msg_num: suntech.fields.get("MSG_NUM").cloned().unwrap_or_default(),
                            msg_type: suntech.fields.get("MSG_TYPE").cloned().unwrap_or_default(),
                            net_status: suntech.fields.get("NET_STATUS").cloned().unwrap_or_default(),
                            odometer_mts: suntech.fields.get("ODOMETER_MTS").cloned().unwrap_or_default(),
                            out_state: suntech.fields.get("OUT_STATE").cloned().unwrap_or_default(),
                            report_map: suntech.fields.get("REPORT_MAP").cloned().unwrap_or_default(),
                            rx_lvl: suntech.fields.get("RX_LVL").cloned().unwrap_or_default(),
                            satellites: suntech.fields.get("SAT").cloned().unwrap_or_default(),
                            speed: suntech.fields.get("SPD").cloned().unwrap_or_default(),
                            speed_time: suntech.fields.get("SPEED_TIME").cloned().unwrap_or_default(),
                            stt_rpt_type: suntech.fields.get("STT_RPT_TYPE").cloned().unwrap_or_default(),
                            total_distance: suntech.fields.get("TOTAL_DISTANCE").cloned().unwrap_or_default(),
                            trip_distance: suntech.fields.get("TRIP_DISTANCE").cloned().unwrap_or_default(),
                            trip_hourmeter: suntech.fields.get("TRIP_HOURMETER").cloned().unwrap_or_default(),
                            volt_backup: suntech.fields.get("VOLT_BACKUP").cloned().unwrap_or_default(),
                            volt_main: suntech.fields.get("VOLT_MAIN").cloned().unwrap_or_default(),
                        }
                    }
                }
                Some(crate::config::siscom::kafka_message::Decoded::Queclink(queclink)) => {
                    crate::models::DecodedData::Queclink {
                        queclink_raw: crate::models::QueclinkRaw {
                            altitude: queclink.fields.get("ALTITUDE").cloned().unwrap_or_default(),
                            cell_id: queclink.fields.get("CELL_ID").cloned().unwrap_or_default(),
                            course: queclink.fields.get("CRS").cloned().unwrap_or_default(),
                            device_id: queclink.fields.get("DEVICE_ID").cloned().unwrap_or_default(),
                            fix: queclink.fields.get("FIX").cloned().unwrap_or_default(),
                            gps_date_time: queclink.fields.get("GPS_DATE_TIME").cloned().unwrap_or_default(),
                            header: queclink.fields.get("HEADER").cloned().unwrap_or_default(),
                            lac: queclink.fields.get("LAC").cloned().unwrap_or_default(),
                            latitude: queclink.fields.get("LAT").cloned().unwrap_or_default(),
                            longitude: queclink.fields.get("LON").cloned().unwrap_or_default(),
                            mcc: queclink.fields.get("MCC").cloned().unwrap_or_default(),
                            mnc: queclink.fields.get("MNC").cloned().unwrap_or_default(),
                            msg_num: queclink.fields.get("MSG_NUM").cloned().unwrap_or_default(),
                            protocol_version: queclink.fields.get("PROTOCOL_VERSION").cloned().unwrap_or_default(),
                            reserved: queclink.fields.get("RESERVED").cloned().unwrap_or_default(),
                            send_date_time: queclink.fields.get("SEND_DATE_TIME").cloned().unwrap_or_default(),
                            speed: queclink.fields.get("SPD").cloned().unwrap_or_default(),
                        }
                    }
                }
                None => {
                    // Si no hay datos decodificados, usar valores por defecto
                    crate::models::DecodedData::Suntech {
                        suntech_raw: crate::models::SuntechRaw::default(),
                    }
                }
            },
            metadata: crate::models::DeviceMetadata {
                bytes: metadata.bytes as i32,
                client_ip: metadata.client_ip.clone(),
                client_port: metadata.client_port as i32,
                decoded_epoch: metadata.decoded_epoch as i64,
                received_epoch: metadata.received_epoch as i64,
                worker_id: metadata.worker_id as i32,
            },
            raw: kafka_msg.raw.clone(),
            uuid: kafka_msg.uuid.clone(),
        };

        Ok(device_message)
    }
}

#[async_trait]
impl MessageConsumer for KafkaConsumerService {
    async fn start_consuming(&self) -> Result<mpsc::UnboundedReceiver<DeviceMessage>> {
        let (tx, rx) = mpsc::unbounded_channel();

        // Suscribirse al topic
        self.consumer.subscribe(&[&self.topic])?;

        info!("🔌 Suscrito al topic Kafka: {}", self.topic);

        // Clonar referencias para la tarea
        let consumer = Arc::clone(&self.consumer);
        let tx_clone = tx.clone();

        // Iniciar tarea de consumo
        tokio::spawn(async move {
            loop {
                match consumer.recv().await {
                    Ok(message) => {
                        if let Some(payload) = message.payload() {
                            match ProstMessage::decode(payload) {
                                Ok(kafka_msg) => {
                                    match Self::kafka_message_to_device_message(&kafka_msg) {
                                        Ok(device_msg) => {
                                            debug!(
                                                "✅ Mensaje protobuf parseado para dispositivo: {}",
                                                device_msg.data.device_id
                                            );

                                            if let Err(e) = tx_clone.send(device_msg) {
                                                error!("Error enviando mensaje al canal: {}", e);
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            error!("❌ Error convirtiendo mensaje protobuf a DeviceMessage: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("❌ Error decodificando mensaje protobuf: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error recibiendo mensaje de Kafka: {}", e);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        });

        Ok(rx)
    }

    async fn disconnect(&self) -> Result<()> {
        info!("🔌 Desconectando de Kafka...");
        // El consumer se desconectará automáticamente al ser dropped
        Ok(())
    }
}

// Nota: Los tests de integración con Kafka requieren un broker real corriendo.
// Para probar SASL authentication manualmente:
//
// 1. Configurar variables de entorno SASL:
//    export KAFKA_SECURITY_PROTOCOL=SASL_PLAINTEXT
//    export KAFKA_SASL_MECHANISM=SCRAM-SHA-256
//    export KAFKA_USERNAME=tu-usuario
//    export KAFKA_PASSWORD=tu-password
//
// 2. Ejecutar la aplicación:
//    cargo run
//
// 3. Verificar logs que muestren:
//    "🔐 Configurando security.protocol: SASL_PLAINTEXT"
//    "🔐 Configurando sasl.mechanism: SCRAM-SHA-256"
//    "🔐 Configurando sasl.username: tu-usuario"
//    "🔐 Configurando sasl.password: [PROTECTED]"