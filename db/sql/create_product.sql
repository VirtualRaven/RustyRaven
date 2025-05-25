INSERT INTO products  (id,      name,price,description,quantity, created,updated,  tags,category,tax_rate)
            VALUES    (DEFAULT, $1,  $2,   $3,         $4,       DEFAULT,NOW(),    $5::product_tag[],$6,$7)
RETURNING id;