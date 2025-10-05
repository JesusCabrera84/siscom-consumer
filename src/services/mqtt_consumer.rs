use anyhow::Result;
use bytes::Bytes;
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use serde_json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::models::DeviceMessage;

#[derive(Clone)]
pub struct MqttConsumerService {
    client: AsyncClient,
    event_loop: Arc<tokio::sync::Mutex<EventLoop>>,
    message_sender: mpsc::UnboundedSender<DeviceMessage>,
}

impl MqttConsumerService {
    pub fn new(
        broker: &str,
        port: u16,
        topic: &str,
        username: Option<&str>,
        password: Option<&str>,
        client_id: &str,
        keep_alive_secs: u64,
        clean_session: bool,
        buffer_size: usize,
    ) -> Result<(Self, mpsc::UnboundedReceiver<DeviceMessage>)> {
        // Configurar opciones MQTT para máximo rendimiento
        let mut mqttoptions = MqttOptions::new(client_id, broker, port);

        // Configuraciones de rendimiento
        mqttoptions.set_keep_alive(Duration::from_secs(keep_alive_secs));
        mqttoptions.set_clean_session(clean_session);
        mqttoptions.set_max_packet_size(1024 * 1024, 1024 * 1024); // 1MB max packet

        // Buffer grande para manejo de ráfagas
        mqttoptions.set_inflight(100); // Múltiples mensajes en vuelo
        mqttoptions.set_request_channel_capacity(buffer_size);
        // mqttoptions.set_notification_channel_capacity(buffer_size); // No disponible en esta versión

        // Autenticación si está configurada
        match (username, password) {
            (Some(user), Some(pass)) => {
                info!("🔐 Configurando credenciales MQTT para usuario: {}", user);
                mqttoptions.set_credentials(user, pass);
            }
            (Some(_), None) => {
                warn!("⚠️ Usuario MQTT configurado pero falta contraseña");
            }
            (None, Some(_)) => {
                warn!("⚠️ Contraseña MQTT configurada pero falta usuario");
            }
            (None, None) => {
                info!("ℹ️ Conectando a MQTT sin autenticación");
            }
        }

        // Crear cliente y event loop
        let (client, event_loop) = AsyncClient::new(mqttoptions, buffer_size);

        // Canal para mensajes procesados
        let (tx, rx) = mpsc::unbounded_channel();

        let service = Self {
            client: client.clone(),
            event_loop: Arc::new(tokio::sync::Mutex::new(event_loop)),
            message_sender: tx,
        };

        // Suscribirse al topic
        tokio::spawn({
            let client = client.clone();
            let topic = topic.to_string();
            async move {
                info!("🔌 Suscribiéndose al topic: {}", topic);

                // Usar QoS 0 para máxima velocidad (fire and forget)
                if let Err(e) = client.subscribe(&topic, QoS::AtMostOnce).await {
                    error!("Error suscribiéndose al topic {}: {}", topic, e);
                }
            }
        });

        Ok((service, rx))
    }

    /// Inicia el loop de consumo de mensajes MQTT
    pub async fn start_consuming(&self) -> Result<()> {
        let mut event_loop = self.event_loop.lock().await;
        let sender = self.message_sender.clone();

        info!("🚀 Iniciando consumo de mensajes MQTT...");

        loop {
            match event_loop.poll().await {
                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    // Procesar mensaje en una tarea separada para no bloquear el loop
                    let payload = publish.payload.clone();
                    let topic = publish.topic.clone();
                    let sender_clone = sender.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::process_message(payload, topic, sender_clone).await {
                            error!("Error procesando mensaje MQTT: {}", e);
                        }
                    });
                }
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    info!("✅ Conectado a broker MQTT");
                }
                Ok(Event::Incoming(Packet::SubAck(_))) => {
                    info!("✅ Suscripción confirmada");
                }
                Ok(Event::Incoming(Packet::PingResp)) => {
                    debug!("📡 Ping response recibido");
                }
                Ok(Event::Outgoing(_)) => {
                    // Eventos salientes (menos importantes para logging)
                }
                Ok(_) => {
                    // Otros eventos
                    debug!("Evento MQTT recibido");
                }
                Err(e) => {
                    error!("Error en MQTT event loop: {}", e);

                    // Intentar reconectar después de un error
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    warn!("Intentando reconectar...");
                }
            }
        }
    }

    /// Procesa un mensaje MQTT individual
    async fn process_message(
        payload: Bytes,
        topic: String,
        sender: mpsc::UnboundedSender<DeviceMessage>,
    ) -> Result<()> {
        // Convertir payload a string
        let message_str = String::from_utf8_lossy(&payload);

        debug!(
            "📨 Mensaje recibido en topic '{}': {} bytes",
            topic,
            payload.len()
        );

        // Intentar parsear como JSON de dispositivo
        match serde_json::from_str::<DeviceMessage>(&message_str) {
            Ok(device_message) => {
                let manufacturer = device_message.get_manufacturer();
                debug!(
                    "✅ Mensaje {:?} parseado para dispositivo: {}",
                    manufacturer, device_message.data.device_id
                );

                // Enviar mensaje procesado al canal
                if let Err(e) = sender.send(device_message) {
                    error!("Error enviando mensaje al canal de procesamiento: {}", e);
                }
            }
            Err(e) => {
                error!("❌ Error parseando JSON de dispositivo: {}", e);
                error!("Payload recibido: {}", message_str);
                // No retornar error para que el loop continúe
            }
        }

        Ok(())
    }

    /// Desconecta del broker MQTT
    pub async fn disconnect(&self) -> Result<()> {
        info!("🔌 Desconectando de MQTT...");

        self.client.disconnect().await?;

        info!("✅ Desconectado de MQTT");
        Ok(())
    }
}
