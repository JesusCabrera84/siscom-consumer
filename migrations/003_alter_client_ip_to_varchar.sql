-- Cambiar el tipo de dato de client_ip de INET a VARCHAR en ambas tablas

-- Tabla communications_suntech
ALTER TABLE communications_suntech 
ALTER COLUMN client_ip TYPE VARCHAR USING client_ip::text;

-- Tabla communications_queclink
ALTER TABLE communications_queclink 
ALTER COLUMN client_ip TYPE VARCHAR USING client_ip::text;

-- Tabla communications_current_state (si existe)
DO $$ 
BEGIN
    IF EXISTS (
        SELECT FROM information_schema.tables 
        WHERE table_name = 'communications_current_state'
    ) THEN
        ALTER TABLE communications_current_state 
        ALTER COLUMN client_ip TYPE VARCHAR USING client_ip::text;
    END IF;
END $$;

