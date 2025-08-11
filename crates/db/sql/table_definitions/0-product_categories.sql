CREATE TABLE IF NOT EXISTS product_categories(
    id integer GENERATED ALWAYS AS IDENTITY NOT NULL,
    name varchar(255) NOT NULL,
    PRIMARY KEY(id)
);