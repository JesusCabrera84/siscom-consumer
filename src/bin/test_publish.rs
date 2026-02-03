use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Configuración del producer igual al que funciona
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "redpanda:9092")
        .set("sasl.mechanism", "SCRAM-SHA-256")
        .set("sasl.username", "siscom-producer")
        .set("sasl.password", "producerpassword")
        .set("security.protocol", "SASL_PLAINTEXT")
        .set("acks", "1")
        .set("linger.ms", "5")
        .set("batch.size", "65536")
        .set("queue.buffering.max.kbytes", "10240")
        .set("compression.type", "lz4")
        .set("message.timeout.ms", "20000")
        .set("request.timeout.ms", "2000")
        .set("retries", "3")
        .set("retry.backoff.ms", "300")
        .set("enable.idempotence", "false")
        .set("delivery.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    // Crear un mensaje protobuf simple (dummy)
    let payload = b"test message";

    // Enviar el mensaje
    let delivery_status = producer
        .send(
            FutureRecord::to("siscom-messages")
                .payload(payload)
                .key("test-key"),
            Duration::from_secs(0),
        )
        .await;

    match delivery_status {
        Ok((partition, offset)) => {
            println!(
                "✅ Mensaje enviado exitosamente a partición {} con offset {}",
                partition, offset
            );
        }
        Err((e, _)) => {
            eprintln!("❌ Error enviando mensaje: {}", e);
        }
    }
}
