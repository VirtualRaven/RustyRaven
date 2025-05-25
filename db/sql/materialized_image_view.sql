CREATE MATERIALIZED VIEW IF NOT EXISTS product_image_info AS
SELECT p.product_id as product_id, array_agg( (p.image_id,(I.C).avg_color, (I.C).variants)::image_info_type ORDER BY p.image_id ASC )::image_info_type[] as images FROM  product_images p
LEFT JOIN 
    (
        SELECT 
          (iv.image_id , avg_color,  ARRAY_AGG( (width,height,variant_id)::image_variant ORDER BY (width*height) ASC))::image_info_type as C
        FROM image_variants iv
        LEFT JOIN (
        	SELECT image_id, avg_color FROM images
        ) AS K
        ON iv.image_id = K.image_id
        GROUP BY (iv.image_id, avg_color)
    ) AS I 
ON p.image_id = ((I.c)::image_info_type).id
GROUP BY p.product_id;


