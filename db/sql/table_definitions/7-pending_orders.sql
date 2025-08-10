-- Active: 1753534940779@@localhost@5433
CREATE TABLE IF NOT EXISTS pending_orders(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp timestamp with time zone NOT NULL DEFAULT (current_timestamp AT TIME ZONE 'UTC')
);