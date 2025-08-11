
UPDATE products  
    SET  
        name=$1,
        price=$2,
        description=$3,
        quantity=$4, 
        updated=NOW()
    where id = $5;