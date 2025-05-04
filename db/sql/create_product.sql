INSERT INTO products  (id,      name,price,description,quantity, created,updated,  tags ,images)
            VALUES    (DEFAULT, $1,  $2,   $3,         $4,       DEFAULT,NOW(),    $5::product_tag[],  $6)
RETURNING id;