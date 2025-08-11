
SELECT product.*, pv.names FROM product_paths pv
RIGHT JOIN 
(
    SELECT price,id,name,description,category,quantity, (i.images) as "images: _"  FROM products p
    LEFT JOIN  product_image_info i
    ON p.id = i.product_id 
    WHERE (id = ANY($1)) 
) AS product
ON pv.id = product.category
