-- =============================================================================
-- PostgreSQL Database Initialization Script
-- =============================================================================
-- This script runs automatically when the PostgreSQL container is first created
-- It sets up essential extensions and database configuration
-- =============================================================================

-- Enable UUID generation extension (required for lighter-auth)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enable cryptographic functions (useful for additional security features)
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Set database timezone to UTC (recommended for consistent timestamps)
ALTER DATABASE lighter_auth SET timezone TO 'UTC';

-- Log the initialization
DO $$
BEGIN
    RAISE NOTICE 'lighter-auth database initialized successfully';
    RAISE NOTICE 'Extensions enabled: uuid-ossp, pgcrypto';
    RAISE NOTICE 'Timezone set to: UTC';
END $$;
