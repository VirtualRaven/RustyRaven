CREATE TABLE IF NOT EXISTS product_images(
    image_id integer NOT NULL,
    product_id integer NOT NULL,
    CONSTRAINT product_images_image_id_fkey FOREIGN key(image_id) REFERENCES images(image_id),
    CONSTRAINT product_images_product_id_fkey FOREIGN key(product_id) REFERENCES products(id)
);