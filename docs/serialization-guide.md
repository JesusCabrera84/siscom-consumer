# Guía de Serialización y Deserialización de Mensajes

## 📋 Introducción

El **SISCOM Consumer** utiliza un sistema robusto de serialización basado en **Protocol Buffers (Protobuf)** para el intercambio eficiente de mensajes GPS entre productores y consumidores. Este documento proporciona toda la información necesaria para trabajar con los mensajes serializados del sistema.

## 🎯 Arquitectura de Mensajes

### Tipos de Mensajes Soportados

El sistema soporta el formato de mensajes Kafka (Protobuf) para streaming de alto rendimiento.

### Fabricantes Soportados

- **SUNTECH**: Dispositivos GPS Suntech con protocolo propietario
- **QUECLINK**: Dispositivos GPS Queclink con protocolo propietario

## 📋 Esquema Protocol Buffers

### Archivo `siscom.proto`

```protobuf
syntax = "proto3";

package siscom.v1;

/* =========================
 * ENUMS
 * ========================= */

enum Vendor {
  VENDOR_UNKNOWN = 0;
  SUNTECH = 1;
  QUECLINK = 2;
}

enum MessageClass {
  MSG_UNKNOWN = 0;
  STATUS = 1;
  EVENT = 2;
  ALERT = 3;
}

/* =========================
 * METADATA
 * ========================= */

message Metadata {
  uint32 worker_id = 1;
  uint64 received_epoch = 2;
  uint64 decoded_epoch = 3;
  uint32 bytes = 4;
  string client_ip = 5;
  uint32 client_port = 6;
}

/* =========================
 * NORMALIZED DATA
 * ========================= */

message NormalizedData {
  string device_id = 1;

  double latitude = 2;
  double longitude = 3;
  double speed = 4;
  double course = 5;

  bool engine_on = 6;
  uint32 satellites = 7;

  MessageClass msg_class = 8;

  uint64 gps_epoch = 9;

  double main_battery_voltage = 10;
  double backup_battery_voltage = 11;

  uint64 odometer_mts = 12;
  uint64 trip_distance_mts = 13;

  // Additional fields that may be present in the normalized data
  map<string, string> additional_fields = 14;
}

/* =========================
 * DECODED MESSAGE (ONEOF)
 * ========================= */

message SuntechDecoded {
  // Suntech-specific decoded fields
  map<string, string> fields = 1;
}

message QueclinkDecoded {
  // Queclink-specific decoded fields
  map<string, string> fields = 1;
}

/* =========================
 * KAFKA CONTRACT MESSAGE
 * ========================= */

message KafkaMessage {
  string uuid = 1;

  // Decoded message as oneof per vendor
  oneof decoded {
    SuntechDecoded suntech = 2;
    QueclinkDecoded queclink = 3;
  }

  // Normalized/homogenized data
  map<string, string> data = 4;

  // Message metadata
  Metadata metadata = 5;

  // Raw payload (original message)
  string raw = 6;
}

/* =========================
 * LEGACY MESSAGE (BACKWARD COMPATIBILITY)
 * ========================= */

message Communication {
  string uuid = 1;

  Vendor vendor = 2;

  NormalizedData data = 3;
  Metadata metadata = 4;

  // Decoded payload as bytes with content type
  // This allows flexibility: today it's JSON, tomorrow it could be
  // vendor-specific protobuf or other binary formats
  bytes decoded_payload = 20;
  string decoded_content_type = 21;

  // Raw payload (original message or hex representation)
  string raw = 22;
}
```

## 🔧 Serialización de Mensajes

### Generación de Código Protobuf

#### Rust

El proyecto utiliza `prost-build` para generar código Rust automáticamente:

```rust
// build.rs
use std::io::Result;

fn main() -> Result<()> {
    prost_build::Config::new()
        .out_dir("src/")
        .compile_protos(&["siscom.proto"], &["."])?;
    Ok(())
}
```

#### Otros Lenguajes

Para generar código en otros lenguajes, utiliza el compilador `protoc`:

```bash
# Python
protoc --python_out=. siscom.proto

# JavaScript/Node.js
protoc --js_out=import_style=commonjs,binary:. siscom.proto

# Java
protoc --java_out=. siscom.proto

# C++
protoc --cpp_out=. siscom.proto
```

### Creación de Mensajes

#### Rust - Creando un Mensaje Kafka

```rust
use siscom::v1::{KafkaMessage, Metadata, SuntechDecoded};
use std::collections::HashMap;

fn create_suntech_message() -> KafkaMessage {
    let mut metadata = Metadata {
        worker_id: 1,
        received_epoch: 1640995200, // 2022-01-01 00:00:00 UTC
        decoded_epoch: 1640995260,  // 1 minute later
        bytes: 150,
        client_ip: "192.168.1.100".to_string(),
        client_port: 1883,
    };

    let mut suntech_data = HashMap::new();
    suntech_data.insert("HEADER".to_string(), "ST300".to_string());
    suntech_data.insert("DEVICE_ID".to_string(), "ABC123".to_string());
    suntech_data.insert("LAT".to_string(), "-12.0464".to_string());
    suntech_data.insert("LON".to_string(), "-77.0428".to_string());
    suntech_data.insert("SPD".to_string(), "45.2".to_string());

    let suntech_decoded = SuntechDecoded {
        fields: suntech_data,
    };

    let mut normalized_data = HashMap::new();
    normalized_data.insert("device_id".to_string(), "ABC123".to_string());
    normalized_data.insert("latitude".to_string(), "-12.0464".to_string());
    normalized_data.insert("longitude".to_string(), "-77.0428".to_string());
    normalized_data.insert("speed".to_string(), "45.2".to_string());

    KafkaMessage {
        uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        decoded: Some(siscom::v1::kafka_message::Decoded::Suntech(suntech_decoded)),
        data: normalized_data,
        metadata: Some(metadata),
        raw: "ST300STT;ABC123;...".to_string(),
    }
}
```

#### Python - Creando un Mensaje Kafka

```python
from siscom_pb2 import KafkaMessage, Metadata, SuntechDecoded

def create_suntech_message():
    # Crear metadata
    metadata = Metadata()
    metadata.worker_id = 1
    metadata.received_epoch = 1640995200
    metadata.decoded_epoch = 1640995260
    metadata.bytes = 150
    metadata.client_ip = "192.168.1.100"
    metadata.client_port = 1883

    # Crear datos decodificados de Suntech
    suntech_decoded = SuntechDecoded()
    suntech_decoded.fields["HEADER"] = "ST300"
    suntech_decoded.fields["DEVICE_ID"] = "ABC123"
    suntech_decoded.fields["LAT"] = "-12.0464"
    suntech_decoded.fields["LON"] = "-77.0428"
    suntech_decoded.fields["SPD"] = "45.2"

    # Crear mensaje principal
    message = KafkaMessage()
    message.uuid = "550e8400-e29b-41d4-a716-446655440000"
    message.suntech.CopyFrom(suntech_decoded)
    message.metadata.CopyFrom(metadata)
    message.raw = "ST300STT;ABC123;..."

    # Datos normalizados
    message.data["device_id"] = "ABC123"
    message.data["latitude"] = "-12.0464"
    message.data["longitude"] = "-77.0428"
    message.data["speed"] = "45.2"

    return message
```

#### JavaScript/Node.js - Creando un Mensaje Kafka

```javascript
const protobuf = require('protobufjs');

async function createSuntechMessage() {
    // Cargar el esquema
    const root = await protobuf.load('siscom.proto');
    const KafkaMessage = root.lookupType('siscom.v1.KafkaMessage');
    const Metadata = root.lookupType('siscom.v1.Metadata');
    const SuntechDecoded = root.lookupType('siscom.v1.SuntechDecoded');

    // Crear metadata
    const metadata = {
        workerId: 1,
        receivedEpoch: 1640995200,
        decodedEpoch: 1640995260,
        bytes: 150,
        clientIp: "192.168.1.100",
        clientPort: 1883
    };

    // Crear datos decodificados
    const suntechDecoded = {
        fields: {
            "HEADER": "ST300",
            "DEVICE_ID": "ABC123",
            "LAT": "-12.0464",
            "LON": "-77.0428",
            "SPD": "45.2"
        }
    };

    // Crear mensaje completo
    const message = {
        uuid: "550e8400-e29b-41d4-a716-446655440000",
        suntech: suntechDecoded,
        data: {
            "device_id": "ABC123",
            "latitude": "-12.0464",
            "longitude": "-77.0428",
            "speed": "45.2"
        },
        metadata: metadata,
        raw: "ST300STT;ABC123;..."
    };

    return message;
}
```

### Serialización a Bytes

#### Rust

```rust
use prost::Message;

let message = create_suntech_message();
let mut buf = Vec::new();
message.encode(&mut buf)?;

// buf ahora contiene los bytes serializados
println!("Mensaje serializado: {} bytes", buf.len());
```

#### Python

```python
message = create_suntech_message()
serialized = message.SerializeToString()
print(f"Mensaje serializado: {len(serialized)} bytes")
```

#### JavaScript/Node.js

```javascript
const message = await createSuntechMessage();
const KafkaMessage = root.lookupType('siscom.v1.KafkaMessage');

const errMsg = KafkaMessage.verify(message);
if (errMsg) throw Error(errMsg);

const serialized = KafkaMessage.encode(message).finish();
console.log(`Mensaje serializado: ${serialized.length} bytes`);
```

## 🔄 Deserialización de Mensajes

### Deserialización desde Bytes

#### Rust

```rust
use prost::Message;
use siscom::v1::KafkaMessage;

fn deserialize_message(data: &[u8]) -> Result<KafkaMessage, prost::DecodeError> {
    KafkaMessage::decode(data)
}

// Uso
let deserialized = deserialize_message(&buf)?;
println!("UUID: {}", deserialized.uuid);
println!("Device ID: {}", deserialized.data.get("device_id").unwrap_or(&"Unknown".to_string()));
```

#### Python

```python
from siscom_pb2 import KafkaMessage

def deserialize_message(data):
    message = KafkaMessage()
    message.ParseFromString(data)
    return message

# Uso
deserialized = deserialize_message(serialized_data)
print(f"UUID: {deserialized.uuid}")
print(f"Device ID: {deserialized.data.get('device_id', 'Unknown')}")
```

#### JavaScript/Node.js

```javascript
function deserializeMessage(data) {
    const KafkaMessage = root.lookupType('siscom.v1.KafkaMessage');
    const message = KafkaMessage.decode(data);
    return KafkaMessage.toObject(message);
}

// Uso
const deserialized = deserializeMessage(serializedData);
console.log(`UUID: ${deserialized.uuid}`);
console.log(`Device ID: ${deserialized.data['device_id'] || 'Unknown'}`);
```

### Procesamiento de Mensajes Deserializados

#### Determinación del Fabricante

```rust
use siscom::v1::kafka_message::Decoded;

fn process_message(message: &KafkaMessage) {
    match &message.decoded {
        Some(Decoded::Suntech(suntech_data)) => {
            println!("Mensaje Suntech detectado");
            process_suntech_data(suntech_data);
        }
        Some(Decoded::Queclink(queclink_data)) => {
            println!("Mensaje Queclink detectado");
            process_queclink_data(queclink_data);
        }
        None => {
            println!("Mensaje sin datos decodificados");
        }
    }
}
```

#### Extracción de Datos Normalizados

```rust
fn extract_location_data(message: &KafkaMessage) -> Option<(f64, f64, f64)> {
    let lat = message.data.get("latitude")?.parse::<f64>().ok()?;
    let lon = message.data.get("longitude")?.parse::<f64>().ok()?;
    let speed = message.data.get("speed")?.parse::<f64>().ok()?;

    Some((lat, lon, speed))
}
```

## 📊 Campos de Datos por Fabricante

### Suntech Raw Fields

| Campo | Descripción | Tipo | Ejemplo |
|-------|-------------|------|---------|
| `HEADER` | Tipo de comando | String | "ST300" |
| `DEVICE_ID` | ID único del dispositivo | String | "ABC123" |
| `LAT` | Latitud | String | "-12.0464" |
| `LON` | Longitud | String | "-77.0428" |
| `SPD` | Velocidad (km/h) | String | "45.2" |
| `CRS` | Curso/dirección | String | "180.0" |
| `GPS_DATE` | Fecha GPS | String | "20231201" |
| `GPS_TIME` | Hora GPS | String | "143022" |
| `FIX` | Estado del GPS fix | String | "1" |
| `SAT` | Número de satélites | String | "8" |
| `ODOMETER_MTS` | Odómetro en metros | String | "123456" |
| `VOLT_MAIN` | Voltaje batería principal | String | "12.5" |
| `VOLT_BACKUP` | Voltaje batería backup | String | "3.8" |

### Queclink Raw Fields

| Campo | Descripción | Tipo | Ejemplo |
|-------|-------------|------|---------|
| `HEADER` | Tipo de comando | String | "+RESP" |
| `DEVICE_ID` | ID único del dispositivo | String | "ABC123" |
| `LAT` | Latitud | String | "-12.0464" |
| `LON` | Longitud | String | "-77.0428" |
| `SPD` | Velocidad (km/h) | String | "45.2" |
| `CRS` | Curso/dirección | String | "180.0" |
| `GPS_DATE_TIME` | Fecha y hora GPS | String | "20231201143022" |
| `FIX` | Estado del GPS fix | String | "1" |
| `SAT` | Número de satélites | String | "8" |
| `PROTOCOL_VERSION` | Versión del protocolo | String | "1.0" |
| `MSG_NUM` | Número de mensaje | String | "123" |

## 🔄 Conversiones y Validaciones

### Conversión de Coordenadas

```rust
fn validate_coordinates(lat: f64, lon: f64) -> Result<(), String> {
    if !(-90.0..=90.0).contains(&lat) {
        return Err("Latitud fuera de rango".to_string());
    }
    if !(-180.0..=180.0).contains(&lon) {
        return Err("Longitud fuera de rango".to_string());
    }
    Ok(())
}
```

### Conversión de Velocidad

```rust
fn parse_speed(speed_str: &str) -> Result<f64, String> {
    speed_str.parse::<f64>()
        .map_err(|_| format!("Velocidad inválida: {}", speed_str))
}
```

### Conversión de Epoch Time

```rust
use chrono::{DateTime, Utc};

fn epoch_to_datetime(epoch: u64) -> DateTime<Utc> {
    DateTime::from_timestamp(epoch as i64, 0).unwrap_or_default()
}

fn datetime_to_epoch(dt: DateTime<Utc>) -> u64 {
    dt.timestamp() as u64
}
```

## 🚀 Integración con Kafka

### Producer Configuration

```rust
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};

fn create_producer(brokers: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation failed")
}
```

### Enviar Mensajes Serializados

```rust
async fn send_message(
    producer: &FutureProducer,
    topic: &str,
    key: &str,
    message: &KafkaMessage,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    message.encode(&mut buf)?;

    let record = FutureRecord::to(topic)
        .key(key)
        .payload(&buf);

    producer.send(record, Duration::from_secs(0)).await?;
    Ok(())
}
```

### Consumer Configuration

```rust
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::config::ClientConfig;

fn create_consumer(brokers: &str, group_id: &str, topics: &[&str]) -> StreamConsumer {
    let mut client_config = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true");

    // Configurar SASL si las variables de entorno están presentes
    if let Ok(security_protocol) = std::env::var("KAFKA_SECURITY_PROTOCOL") {
        client_config = client_config.set("security.protocol", security_protocol);
    }

    if let Ok(sasl_mechanism) = std::env::var("KAFKA_SASL_MECHANISM") {
        client_config = client_config.set("sasl.mechanism", sasl_mechanism);
    }

    if let Ok(username) = std::env::var("KAFKA_USERNAME") {
        client_config = client_config.set("sasl.username", username);
    }

    if let Ok(password) = std::env::var("KAFKA_PASSWORD") {
        client_config = client_config.set("sasl.password", password);
    }

    let consumer: StreamConsumer = client_config
        .create()
        .expect("Consumer creation failed");

    consumer.subscribe(topics).expect("Can't subscribe to topics");
    consumer
}
```

#### Consumer con SASL Authentication

```rust
// Configuración de consumer con SASL SCRAM-SHA-256
fn create_secure_consumer(brokers: &str, group_id: &str, topics: &[&str]) -> StreamConsumer {
    let mut binding = ClientConfig::new();
    let client_config = binding
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        // Configuración SASL
        .set("security.protocol", "SASL_PLAINTEXT")
        .set("sasl.mechanism", "SCRAM-SHA-256")
        .set("sasl.username", "siscom-consumer")
        .set("sasl.password", "consumerpassword");

    let consumer: StreamConsumer = client_config
        .create()
        .expect("Consumer creation failed");

    consumer.subscribe(topics).expect("Can't subscribe to topics");
    consumer
}
```

### Procesar Mensajes Recibidos

```rust
async fn process_messages(consumer: StreamConsumer) {
    loop {
        match consumer.recv().await {
            Ok(message) => {
                match KafkaMessage::decode(message.payload().unwrap()) {
                    Ok(decoded_message) => {
                        process_message(&decoded_message).await;
                    }
                    Err(e) => {
                        eprintln!("Error deserializando mensaje: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error recibiendo mensaje: {}", e);
            }
        }
    }
}
```

## 🧪 Testing y Validación

### Validación de Mensajes

```rust
fn validate_message(message: &KafkaMessage) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Validar UUID
    if message.uuid.is_empty() {
        errors.push("UUID no puede estar vacío".to_string());
    }

    // Validar que tenga datos decodificados
    if message.decoded.is_none() {
        errors.push("Mensaje debe tener datos decodificados".to_string());
    }

    // Validar coordenadas si están presentes
    if let Some((lat, lon, _)) = extract_location_data(message) {
        if let Err(e) = validate_coordinates(lat, lon) {
            errors.push(e);
        }
    }

    // Validar metadata
    if let Some(metadata) = &message.metadata {
        if metadata.client_ip.is_empty() {
            errors.push("IP del cliente no puede estar vacía".to_string());
        }
    } else {
        errors.push("Metadata es requerida".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

### Tests Unitarios

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let message = create_suntech_message();
        let mut buf = Vec::new();
        message.encode(&mut buf).unwrap();

        let deserialized = KafkaMessage::decode(&buf).unwrap();
        assert_eq!(message.uuid, deserialized.uuid);
    }

    #[test]
    fn test_coordinate_validation() {
        assert!(validate_coordinates(0.0, 0.0).is_ok());
        assert!(validate_coordinates(91.0, 0.0).is_err());
        assert!(validate_coordinates(0.0, 181.0).is_err());
    }
}
```

## 🔐 Autenticación SASL en Kafka

### Configuración SASL

Para usar autenticación SASL con Kafka, configura las siguientes variables de entorno:

```bash
export KAFKA_SECURITY_PROTOCOL=SASL_PLAINTEXT
export KAFKA_SASL_MECHANISM=SCRAM-SHA-256
export KAFKA_USERNAME=siscom-consumer
export KAFKA_PASSWORD=consumerpassword
```

### Verificación de Configuración

Cuando inicies la aplicación, deberías ver logs como:

```
🔐 Configurando security.protocol: SASL_PLAINTEXT
🔐 Configurando sasl.mechanism: SCRAM-SHA-256
🔐 Configurando sasl.username: siscom-consumer
🔐 Configurando sasl.password: [PROTECTED]
```

### Requisitos del Sistema

- **librdkafka** compilado con soporte SASL (`libsasl2` o `openssl`)
- Broker Kafka/Redpanda con autenticación SASL habilitada
- Credenciales válidas en el servidor Kafka

### Troubleshooting SASL

**Error: "No provider for SASL mechanism SCRAM-SHA-256"**
- librdkafka no fue compilado con soporte SASL
- Instalar `libsasl2-dev` y recompilar rdkafka

**Error: "SASL authentication failed"**
- Verificar credenciales (username/password)
- Verificar que el mecanismo SASL esté habilitado en el broker
- Revisar configuración del broker Kafka

## ⚠️ Consideraciones Importantes

### Versionado de Esquemas

- Los esquemas Protobuf son **backward compatible**
- Nuevos campos deben tener números de tag únicos
- Nunca cambiar el tipo de un campo existente
- Usar `deprecated = true` para campos obsoletos

### Manejo de Errores

```rust
fn safe_deserialize(data: &[u8]) -> Result<KafkaMessage, String> {
    match KafkaMessage::decode(data) {
        Ok(message) => {
            match validate_message(&message) {
                Ok(_) => Ok(message),
                Err(errors) => Err(format!("Validación fallida: {:?}", errors)),
            }
        }
        Err(e) => Err(format!("Error de deserialización: {}", e)),
    }
}
```

### Rendimiento

- **Compresión**: Considera comprimir mensajes grandes
- **Batching**: Agrupa mensajes para envío eficiente
- **Pooling**: Reutiliza buffers para evitar allocations
- **Async processing**: Usa procesamiento asíncrono para alto throughput

### Seguridad

- **Validación**: Siempre valida mensajes entrantes
- **Sanitización**: Limpia datos antes de procesar
- **Rate limiting**: Implementa límites de tasa para prevenir abuso
- **Authentication**: Usa autenticación en conexiones Kafka

## 📚 Recursos Adicionales

- [Protocol Buffers Documentation](https://developers.google.com/protocol-buffers)
- [Rust Prost Crate](https://docs.rs/prost/latest/prost/)
- [Kafka Rust Client](https://docs.rs/rdkafka/latest/rdkafka/)
- [Protobuf JavaScript Guide](https://github.com/protobufjs/protobuf.js)

---

**Última actualización**: Enero 2026
**Versión del esquema**: v1.0