SELECT product_images.image_id,variant_ids FROM product_images 
JOIN
    (SELECT 
        image_id, 
        array_agg(
            variant_id 
            ORDER BY 
                (width * height) 
            DESC  
        ) as variant_ids
        FROM image_variants GROUP BY image_id) as I
ON (product_images.image_id = I.image_id)
WHERE product_id=$1