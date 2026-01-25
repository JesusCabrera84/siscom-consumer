.PHONY: build run test fmt clippy clean dev setup docker-build docker-run docker-mqtt docker-kafka migrate help

# Default target
help:
	@echo "Available targets:"
	@echo "  build         - Build release version"
	@echo "  run           - Run in development mode"
	@echo "  dev           - Run with debug logs"
	@echo "  test          - Run tests"
	@echo "  fmt           - Format code"
	@echo "  clippy        - Run clippy linter"
	@echo "  clean         - Clean build artifacts"
	@echo "  setup         - Setup development environment"
	@echo "  docker-build  - Build Docker image"
	@echo "  docker-mqtt   - Run with Docker Compose (MQTT mode)"
	@echo "  docker-kafka  - Run with Docker Compose (Kafka mode)"
	@echo "  migrate       - Run database migrations"
	@echo "  help          - Show this help"

# Build release version
build:
	cargo build --release

# Run in development mode
run:
	@if [ ! -f .env ]; then \
		echo "Error: .env file not found. Copy .env.template and configure it."; \
		exit 1; \
	fi
	RUST_LOG=info cargo run

# Run in development with debug logs
dev:
	@if [ ! -f .env ]; then \
		echo "Error: .env file not found. Copy .env.template and configure it."; \
		exit 1; \
	fi
	RUST_LOG=debug cargo run

# Run tests
test:
	cargo test

# Format code
fmt:
	cargo fmt

# Run clippy linter
clippy:
	cargo clippy -- -D warnings

# Clean build artifacts
clean:
	cargo clean

# Setup development environment
setup:
	@echo "Setting up development environment..."
	@if [ ! -f .env ]; then \
		echo "Creating .env from template..."; \
		cp .env.template .env; \
		echo "✅ .env created. Please edit it with your configuration."; \
	else \
		echo "✅ .env already exists."; \
	fi
	cargo build

# Docker targets
docker-build:
	docker build -t siscom-consumer .

docker-run: docker-build
	@if [ ! -f .env ]; then \
		echo "Error: .env file not found. Copy .env.template and configure it."; \
		exit 1; \
	fi
	docker run --env-file .env siscom-consumer

docker-mqtt:
	@if [ ! -f .env ]; then \
		echo "Error: .env file not found. Copy .env.template and configure it."; \
		exit 1; \
	fi
	BROKER_TYPE=mqtt docker-compose --profile mqtt up

docker-kafka:
	@if [ ! -f .env ]; then \
		echo "Error: .env file not found. Copy .env.template and configure it."; \
		exit 1; \
	fi
	BROKER_TYPE=kafka docker-compose --profile kafka up

# Database migrations
migrate:
	sqlx migrate run
