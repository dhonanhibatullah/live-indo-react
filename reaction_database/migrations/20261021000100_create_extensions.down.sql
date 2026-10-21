-- Refuse to remove PostGIS while other database objects still depend on it.
DROP EXTENSION IF EXISTS postgis RESTRICT;
