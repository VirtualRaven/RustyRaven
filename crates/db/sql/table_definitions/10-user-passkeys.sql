
CREATE TABLE IF NOT EXISTS user_passkeys (
    user_id UUID NOT NULL,
    keyid bytea,
    passkey bytea,
    CONSTRAINT user_id_fkey FOREIGN key(user_id) REFERENCES users(id),
    CONSTRAINT non_empty_keyid CHECK (keyid is NOT NULL),
    CONSTRAINT non_empty_passkey CHECK (passkey is NOT NULL)
);