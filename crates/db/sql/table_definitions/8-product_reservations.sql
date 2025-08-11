CREATE TABLE IF NOT EXISTS product_reservations(
    reservation_id UUID NOT NULL,
    product_id integer NOT NULL,
    quantity integer NOT NULL,
    CONSTRAINT reservation_id_fkey FOREIGN key(reservation_id) REFERENCES pending_orders(id),
    CONSTRAINT product_id_fkey FOREIGN key(product_id) REFERENCES products(id),
    CONSTRAINT positive_quantity CHECK (quantity > 0)
);