CREATE TABLE products (
    id integer PRIMARY KEY generated always as identity,
    name VARCHAR(100),
    price INTEGER CONSTRAINT positive_price CHECK (price > 0),
    description text,
    created TIMESTAMP WITH TIME ZONE,
    updated TIMESTAMP WITH TIME ZONE,
    tags product_tag[],
    images VARCHAR(200)[]
)