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
MQTT → Consumer → Processor → [POI/Geofence Logic] → Kafka + PostgreSQL
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
