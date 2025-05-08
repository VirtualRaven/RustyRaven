
CREATE TABLE images (
    image_id integer PRIMARY KEY generated always as identity,
    avg_color VARCHAR(6) NOT NULL
)