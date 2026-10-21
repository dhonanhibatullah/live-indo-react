-- Refuse to remove extensions while other database objects still depend on them.
DROP EXTENSION IF EXISTS pg_trgm RESTRICT;
DROP EXTENSION IF EXISTS postgis RESTRICT;
