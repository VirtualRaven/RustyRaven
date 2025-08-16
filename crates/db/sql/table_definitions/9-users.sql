
CREATE TABLE IF NOT EXISTS users(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar(100),
    created timestamp with time zone NOT NULL DEFAULT (current_timestamp AT TIME ZONE 'UTC'),
    updated timestamp with time zone NOT NULL DEFAULT (current_timestamp AT TIME ZONE 'UTC'),
    last_login timestamp with time zone NOT NULL DEFAULT (current_timestamp AT TIME ZONE 'UTC')
);