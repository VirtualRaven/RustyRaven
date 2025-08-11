
CREATE TABLE product_images (
    image_id integer REFERENCES images(image_id),
    product_id INTEGER REFERENCES products(id),
    CONSTRAINT  unique_image_product_association UNIQUE(image_id,product_id)
)