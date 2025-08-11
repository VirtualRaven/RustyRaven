CREATE TABLE IF NOT EXISTS products(
    id integer GENERATED ALWAYS AS IDENTITY NOT NULL,
    name varchar(100) NOT NULL,
    price integer NOT NULL,
    description text NOT NULL,
    quantity integer,
    created timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    tax_rate integer NOT NULL,
    category integer NOT NULL,
    PRIMARY KEY(id),
    CONSTRAINT products_category_fkey FOREIGN key(category) REFERENCES product_categories(id),
    CONSTRAINT positive_price CHECK (price > 0),
    CONSTRAINT positive_quantity CHECK ((quantity IS NULL) OR (quantity >= 0)),
    CONSTRAINT reasonable_tax_rate CHECK ((tax_rate = 0) OR (tax_rate = 6) OR (tax_rate = 12) OR (tax_rate = 25))
);