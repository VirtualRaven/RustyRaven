SELECT price,id,name, (i.images) as "images: _"  FROM products p
LEFT JOIN  product_image_info i
ON p.id = i.product_id 
WHERE (quantity!=0 and category=ANY($1)) LIMIT $2 ;
