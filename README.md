# Tracking Consumer Rust

High-performance MQTT to Kafka consumer for GPS tracking data processing.

## Features

- 🚀 **High Performance**: 5-10x faster than Python version
- 🔄 **Async Processing**: True parallel processing with Tokio
- 🛡️ **Type Safety**: Compile-time error prevention
- 📊 **Real-time**: POI and Geofence processing
- 🔌 **Reliable**: Auto-reconnection for MQTT and Kafka
- 📈 **Scalable**: Handles thousands of messages per second

## Quick Start

```bash
# 1. Clone and build
cargo build --release

# 2. Copy and configure
cp .env.template .env
# Edit .env with your settings

# 3. Run
cargo run --release
```

## Architecture

```
MQTT → Consumer → Kafka + PostgreSQL
```

## How It Works

**Tracking Consumer Rust** is a high-performance service designed to consume GPS tracking data from an MQTT broker and forward the raw data to Kafka and PostgreSQL for further use and analytics. The logic for Points of Interest (POI) and Geofence evaluation is handled by a separate microservice that consumes from Kafka.

### Execution Flow

1. **Startup & Configuration**
   - Loads configuration from `config/app.toml` or environment.
   - Initializes logging and prepares for graceful shutdown.

2. **Service Initialization**
   - Connects to PostgreSQL (for persistent storage).
   - Configures the Kafka producer (for streaming data).
   - Sets up the MQTT consumer (to receive GPS messages).
   - Initializes the message processor (handles batching and dispatch).

3. **Main Processing Loop**
   - Starts the MQTT consumer in the background, which receives messages and pushes them to an internal channel.
   - The message processor consumes messages from the channel and batches results.
   - Data is sent to Kafka and PostgreSQL.
   - Health checks and statistics are periodically logged.

4. **Graceful Shutdown**
   - On receiving a shutdown signal, the application flushes all buffers, closes Kafka and MQTT connections, and ensures all data is persisted.

### Execution Diagram

```mermaid
flowchart TD
    A[MQTT Broker] -->|GPS Messages| B(MQTT Consumer)
    B --> C[Internal Channel]
    C --> D[Message Processor]
    D --> E[Kafka Producer]
    D --> F[PostgreSQL]
    E --> G[Kafka Broker]
    F --> H[Database]
    G --> I[POI/Geofence Microservice]
```

## Performance

- **Throughput**: 1000-5000 msg/s
- **Memory**: ~10-20 MB
- **Latency**: <10ms processing time
- **CPU**: ~5-15% on modern hardware

## Configuration

Edit `config/app.toml` for detailed configuration.

## Development

```bash
# Run tests
cargo test

# Run with logs
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Check code
cargo clippy
```
