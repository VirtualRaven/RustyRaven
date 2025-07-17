CREATE TABLE IF NOT EXISTS image_variants(
    variant_id integer GENERATED ALWAYS AS IDENTITY NOT NULL,
    image_id integer NOT NULL,
    width integer NOT NULL,
    height integer NOT NULL,
    PRIMARY KEY(variant_id),
    CONSTRAINT image_variants_image_id_fkey FOREIGN key(image_id) REFERENCES images(image_id),
    CONSTRAINT positive_width CHECK (width > 0),
    CONSTRAINT positive_height CHECK (height > 0)
);