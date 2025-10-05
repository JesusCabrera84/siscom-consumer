-- Crear tabla communications_suntech
CREATE TABLE IF NOT EXISTS communications_suntech (
    id BIGSERIAL PRIMARY KEY,
    uuid VARCHAR NOT NULL,
    device_id VARCHAR NOT NULL,
    backup_battery_voltage NUMERIC,
    backup_battery_percent NUMERIC,
    cell_id VARCHAR,
    course NUMERIC,
    delivery_type VARCHAR,
    engine_status VARCHAR,
    firmware VARCHAR,
    fix_status VARCHAR,
    gps_datetime TIMESTAMP WITHOUT TIME ZONE,
    gps_epoch BIGINT,
    idle_time INTEGER,
    lac VARCHAR,
    latitude NUMERIC(10, 7),
    longitude NUMERIC(10, 7),
    main_battery_voltage NUMERIC,
    mcc VARCHAR,
    mnc VARCHAR,
    model VARCHAR,
    msg_class VARCHAR,
    msg_counter INTEGER,
    alert_type VARCHAR,
    network_status VARCHAR,
    odometer BIGINT,
    rx_lvl INTEGER,
    satellites INTEGER,
    speed NUMERIC,
    speed_time INTEGER,
    total_distance BIGINT,
    trip_distance BIGINT,
    trip_hourmeter INTEGER,
    bytes_count INTEGER,
    client_ip VARCHAR,
    client_port INTEGER,
    decoded_epoch BIGINT,
    received_epoch BIGINT,
    raw_message TEXT,
    received_at TIMESTAMP WITHOUT TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT NOW()
);

-- Índices para optimizar consultas frecuentes
CREATE INDEX IF NOT EXISTS idx_communications_suntech_device_id ON communications_suntech(device_id);
CREATE INDEX IF NOT EXISTS idx_communications_suntech_gps_datetime ON communications_suntech(gps_datetime);
CREATE INDEX IF NOT EXISTS idx_communications_suntech_received_at ON communications_suntech(received_at);
CREATE INDEX IF NOT EXISTS idx_communications_suntech_uuid ON communications_suntech(uuid);

-- Índice compuesto para consultas de dispositivo por fecha
CREATE INDEX IF NOT EXISTS idx_communications_suntech_device_date ON communications_suntech(device_id, gps_datetime);

-- Comentarios de la tabla
COMMENT ON TABLE communications_suntech IS 'Tabla para almacenar comunicaciones de dispositivos Suntech';
COMMENT ON COLUMN communications_suntech.uuid IS 'UUID único del mensaje';
COMMENT ON COLUMN communications_suntech.device_id IS 'ID del dispositivo que envió el mensaje';
COMMENT ON COLUMN communications_suntech.gps_datetime IS 'Fecha y hora del GPS del dispositivo';
COMMENT ON COLUMN communications_suntech.latitude IS 'Latitud del dispositivo';
COMMENT ON COLUMN communications_suntech.longitude IS 'Longitud del dispositivo';
COMMENT ON COLUMN communications_suntech.raw_message IS 'Mensaje crudo original';
COMMENT ON COLUMN communications_suntech.received_at IS 'Fecha y hora de recepción del mensaje';
COMMENT ON COLUMN communications_suntech.created_at IS 'Fecha y hora de creación del registro';
