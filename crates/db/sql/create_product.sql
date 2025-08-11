INSERT INTO products  (id,      name,price,description,quantity, created,updated,  category,tax_rate)
            VALUES    (DEFAULT, $1,  $2,   $3,         $4,       DEFAULT,NOW(),    $5,$6)
RETURNING id;