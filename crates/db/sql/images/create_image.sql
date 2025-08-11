
with inserted_id AS (
    INSERT INTO images (avg_color) VALUES ($1)
    RETURNING image_id
)

INSERT INTO image_variants (image_id,width,height) VALUES 
    ( (SELECT * FROM inserted_id), $2, $3 ),
    ( (SELECT * FROM inserted_id), $4, $5 ),
    ( (SELECT * FROM inserted_id), $6, $7 ),
    ( (SELECT * FROM inserted_id), $8, $9 )
RETURNING image_id, variant_id