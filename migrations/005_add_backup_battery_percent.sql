-- Agregar campo backup_battery_percent a las tablas existentes

-- Tabla communications_suntech
ALTER TABLE communications_suntech 
ADD COLUMN IF NOT EXISTS backup_battery_percent NUMERIC;

-- Tabla communications_queclink
ALTER TABLE communications_queclink 
ADD COLUMN IF NOT EXISTS backup_battery_percent NUMERIC;

-- Tabla communications_current_state (si existe)
DO $$ 
BEGIN
    IF EXISTS (
        SELECT FROM information_schema.tables 
        WHERE table_name = 'communications_current_state'
    ) THEN
        ALTER TABLE communications_current_state 
        ADD COLUMN IF NOT EXISTS backup_battery_percent NUMERIC;
    END IF;
END $$;

-- Comentarios
COMMENT ON COLUMN communications_suntech.backup_battery_percent IS 'Porcentaje de batería de respaldo del dispositivo';
COMMENT ON COLUMN communications_queclink.backup_battery_percent IS 'Porcentaje de batería de respaldo del dispositivo';

