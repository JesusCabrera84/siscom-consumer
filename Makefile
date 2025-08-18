.PHONY: build run test fmt clippy clean dev setup

# Build release version
build:
	cargo build --release

# Run in development mode
run:
	RUST_LOG=info cargo run

# Run in development with debug logs
dev:
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
	cp .env.template .env
	cargo build

# Run with Docker (future)
docker-run:
	docker build -t tracking-consumer .
	docker run tracking-consumer

# Database migrations (future)
migrate:
	sqlx migrate run
