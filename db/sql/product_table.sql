-- Active: 1746291728500@@127.0.0.1@5432@sjf
CREATE TABLE products (
    id integer PRIMARY KEY generated always as identity,
    name VARCHAR(100) NOT NULL,
    price INTEGER CONSTRAINT positive_price CHECK (price > 0),
    description text not NULL,
    quantity integer CONSTRAINT positive_quantity CHECK ( (quantity IS NULL) OR (quantity >0) ),
    created TIMESTAMP WITH TIME ZONE DEFAULT   CURRENT_TIMESTAMP,
    updated TIMESTAMP WITH TIME ZONE DEFAULT   CURRENT_TIMESTAMP,
    tags product_tag[],
    images VARCHAR(200)[]
)