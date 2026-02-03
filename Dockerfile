# Dockerfile para SISCOM Consumer
# Multi-stage build para optimizar el tamaño de la imagen final

# Etapa 1: Builder - Compilación del proyecto
FROM rust:bookworm AS builder

# Instalar dependencias del sistema necesarias para compilar
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Crear directorio de trabajo
WORKDIR /app

# Copiar archivos de configuración y código fuente
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY assets/ ./assets/

# Limpiar cache para forzar recompilación con CMAKE_ARGS
RUN cargo clean

# Compilar la aplicación en modo release
RUN echo "🔨 Compilando aplicación con SCRAM support..."
RUN cargo build --release

# Etapa 2: Runtime - Imagen final minimalista
FROM debian:bookworm

# Instalar dependencias de runtime necesarias
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Crear usuario no-root para seguridad
RUN groupadd -r siscom && useradd -r -g siscom siscom

# Crear directorio de trabajo
WORKDIR /app

# Copiar el binario compilado desde la etapa builder
COPY --from=builder /app/target/release/siscom-consumer /app/siscom-consumer

# Verificar que el binario se copió correctamente
RUN ls -la /app/siscom-consumer
RUN chmod +x /app/siscom-consumer

# Copiar assets si existen
COPY --from=builder /app/assets/ ./assets/

# Cambiar propiedad de los archivos al usuario siscom
RUN chown -R siscom:siscom /app

# Cambiar al usuario no-root
USER siscom

# Variables de entorno por defecto
ENV RUST_LOG=info

# Health check para verificar que el servicio esté funcionando
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pidof siscom-consumer || exit 1

# Comando por defecto
CMD ["/app/siscom-consumer"]