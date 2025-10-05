-- Agregar campo alert_type a las tablas existentes

-- Tabla communications_suntech
ALTER TABLE communications_suntech 
ADD COLUMN IF NOT EXISTS alert_type VARCHAR;

-- Tabla communications_queclink
ALTER TABLE communications_queclink 
ADD COLUMN IF NOT EXISTS alert_type VARCHAR;

-- Tabla communications_current_state (si existe)
DO $$ 
BEGIN
    IF EXISTS (
        SELECT FROM information_schema.tables 
        WHERE table_name = 'communications_current_state'
    ) THEN
        ALTER TABLE communications_current_state 
        ADD COLUMN IF NOT EXISTS alert_type VARCHAR;
    END IF;
END $$;

-- Comentarios
COMMENT ON COLUMN communications_suntech.alert_type IS 'Tipo de alerta del dispositivo';
COMMENT ON COLUMN communications_queclink.alert_type IS 'Tipo de alerta del dispositivo';

