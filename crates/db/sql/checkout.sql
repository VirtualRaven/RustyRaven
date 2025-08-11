SELECT id as product_id, images[1].id as image_id, images[1].variants[array_upper(images[1].variants,1)].variant as image_variant_id, name, price, reserved_quantity as ordered_quantity, tax_rate from product_image_info as I

JOIN (
    SELECT id,name,price,reserved_quantity, tax_rate from products as P
    JOIN
    (
        SELECT quantity as reserved_quantity,product_id from product_reservations where reservation_id = $1
    ) AS R
    ON p.id = R.product_id
)
ON I.product_id = id