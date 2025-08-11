-- Active: 1754908662837@@127.0.0.1@5432@sjf
CREATE TABLE IF NOT EXISTS pending_orders(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp timestamp with time zone NOT NULL DEFAULT (current_timestamp AT TIME ZONE 'UTC')
);