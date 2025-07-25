SELECT 
    p.id,p.name,p.price,p.description,p.quantity,p.created,p.updated, image_ids, tax_rate,category
from products p 
LEFT JOIN 
    (
        SELECT 
            product_id, ARRAY_AGG(image_id) as image_ids 
        FROM product_images 
        GROUP BY product_id
    ) AS I 
ON p.id = i.product_id 
WHERE category = $1
ORDER BY p.name ASC