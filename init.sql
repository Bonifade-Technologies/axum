-- Initialize database for Axum application
-- This script runs when the PostgreSQL container starts for the first time

-- Create database if it doesn't exist
SELECT 'CREATE DATABASE axum_auth'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'axum_auth')\gexec

-- Connect to the database
\c axum_auth;

-- Create extensions if needed
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "citext";

-- Set timezone
SET timezone = 'UTC';
