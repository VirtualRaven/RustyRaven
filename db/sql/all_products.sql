SELECT 
    p.id,p.name,p.price,p.description,p.quantity,p.created,p.updated,p.tags as "product_tag: _", image_ids, tax_rate
from products p 
LEFT JOIN 
    (
        SELECT 
            product_id, ARRAY_AGG(image_id) as image_ids 
        FROM product_images 
        GROUP BY product_id
    ) AS I 
ON p.id = i.product_id 
ORDER BY p.name ASC