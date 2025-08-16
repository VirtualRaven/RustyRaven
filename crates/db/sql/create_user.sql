
    INSERT INTO users (id,name,updated) VALUES ($1, $2, current_timestamp AT TIME ZONE 'UTC') 
    ON CONFLICT(id) DO update SET updated=excluded.updated, name=excluded.name;