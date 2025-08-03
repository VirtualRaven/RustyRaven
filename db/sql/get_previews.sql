SELECT product.price, product.id, product.name,"images: _", pv.names FROM product_paths pv
RIGHT JOIN 
(
    SELECT price,id,name, created, category, (i.images) as "images: _" 
    FROM products p
    LEFT JOIN  product_image_info i
    ON p.id = i.product_id 
    WHERE ( (quantity!=0 or quantity IS NULL) and category=ANY($1)) 
) AS product
ON pv.id = product.category
ORDER BY created DESC
LIMIT $2 
