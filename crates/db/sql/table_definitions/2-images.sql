CREATE TABLE IF NOT EXISTS images(
    image_id integer GENERATED ALWAYS AS IDENTITY NOT NULL,
    avg_color varchar(6) NOT NULL,
    PRIMARY KEY(image_id)
);