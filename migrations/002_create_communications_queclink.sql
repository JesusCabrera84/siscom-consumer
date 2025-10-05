-- Crear tabla communications_queclink
CREATE TABLE IF NOT EXISTS communications_queclink (
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
CREATE INDEX IF NOT EXISTS idx_communications_queclink_device_id ON communications_queclink(device_id);
CREATE INDEX IF NOT EXISTS idx_communications_queclink_gps_datetime ON communications_queclink(gps_datetime);
CREATE INDEX IF NOT EXISTS idx_communications_queclink_received_at ON communications_queclink(received_at);
CREATE INDEX IF NOT EXISTS idx_communications_queclink_uuid ON communications_queclink(uuid);

-- Índice compuesto para consultas de dispositivo por fecha
CREATE INDEX IF NOT EXISTS idx_communications_queclink_device_date ON communications_queclink(device_id, gps_datetime);

-- Comentarios de la tabla
COMMENT ON TABLE communications_queclink IS 'Tabla para almacenar comunicaciones de dispositivos Queclink';
COMMENT ON COLUMN communications_queclink.uuid IS 'UUID único del mensaje';
COMMENT ON COLUMN communications_queclink.device_id IS 'ID del dispositivo que envió el mensaje';
COMMENT ON COLUMN communications_queclink.gps_datetime IS 'Fecha y hora del GPS del dispositivo';
COMMENT ON COLUMN communications_queclink.latitude IS 'Latitud del dispositivo';
COMMENT ON COLUMN communications_queclink.longitude IS 'Longitud del dispositivo';
COMMENT ON COLUMN communications_queclink.raw_message IS 'Mensaje crudo original';
COMMENT ON COLUMN communications_queclink.received_at IS 'Fecha y hora de recepción del mensaje';
COMMENT ON COLUMN communications_queclink.created_at IS 'Fecha y hora de creación del registro';

