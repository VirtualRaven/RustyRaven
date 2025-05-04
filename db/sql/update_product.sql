
UPDATE products  
    SET  
        name=$1,
        price=$2,
        description=$3,
        quantity=$4, 
        updated=NOW(),  
        tags=$5::product_tag[] ,
        images=$6
    where id = $7;