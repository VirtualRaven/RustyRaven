
SELECT product.*, pv.names FROM product_paths pv
RIGHT JOIN 
(
    SELECT price,id,name,description, (i.images) as "images: _"  FROM products p
    LEFT JOIN  product_image_info i
    ON p.id = i.product_id 
    WHERE (id = $1) LIMIT 1
) AS product
ON pv.id = product.id
