use std::str::FromStr;

use sjf_api::checkout::CheckoutRequest;
use sqlx::{query, query_as,query_file,query_file_as };
use tracing::info;
use crate::postgres::POOL;


#[derive(thiserror::Error,Debug)]
 pub enum CheckoutError 
 {
    #[error("Database error {0}")]
    Sql(#[from] sqlx::Error),
    #[error("Uuid error {0}")]
    Uuid(#[from] sqlx::types::uuid::Error),
    #[error("Product id {0} doesn't exists")]
    ProductNotFound(u32),
    #[error("Empty order")]
    EmptyOrder
}




pub async fn make_reservation(req: CheckoutRequest ) -> Result<String,CheckoutError>
{

    if req.order.is_empty()
    {
        return  Err(CheckoutError::EmptyOrder);
    }

    let mut tx = crate::postgres::POOL.get().unwrap().begin().await?;

    let reservation_id = query!("INSERT INTO  pending_orders VALUES( default,default) RETURNING id")
    .fetch_one(&mut *tx)
    .await?.id;
    

    for (id,quantity) in &req.order
    {
        let id = *id as i32;
        let quanity = *quantity as i32;
        query!("SELECT (1) as exists from products where id=$1",id)
        .fetch_one(&mut *tx)
        .await?;

        query!("UPDATE products  SET  quantity=(quantity-$1) WHERE (quantity IS NOT NULL and id=$2)",quanity,id )
        .execute(&mut *tx)
        .await?;

        query!("INSERT INTO product_reservations(reservation_id,product_id,quantity) VALUES($1,$2,$3)",reservation_id,id,quanity)
        .execute(&mut *tx)
        .await?;

    }

    tx.commit().await?;

    Ok(reservation_id.to_string())
}   


pub(crate) async fn undo_old_reservations() -> Result<(),CheckoutError>
{
    let mut expired_transactions = query!("SELECT id FROM pending_orders where timestamp  < now() - interval '35min'")
    .fetch(POOL.get().unwrap());


    use futures_util::TryStreamExt;
    while let Some(r) = expired_transactions.try_next().await?
    {
        undo_reservation(r.id.into()).await?;
    }

    Ok(())
}

pub struct OrderItem 
{
    pub product_id: u32,
    pub image_path: Option<String>,
    pub name: String,
    pub price: u32,
    pub ordered_quantity: u32,
    pub tax_rate: u32
}

pub async fn get_order(uuid: &str) -> Result<Vec<OrderItem>,CheckoutError> 
{
    let uuid = sqlx::types::Uuid::from_str(uuid)?;

    let items= query_file!("sql/checkout.sql",uuid)
    .fetch_all(POOL.get().unwrap())
    .await?;

    let res = items.into_iter().map(|i| {
        OrderItem {
            product_id: i.product_id as u32,
            image_path: {
                match (i.image_id, i.image_variant_id)
                {
                    (Some(id),Some(variant)) => Some(format!("/images/{}/{}",id,variant)),
                    _ => None
                }
            },
            name: i.name,
            price: i.price as u32,
            ordered_quantity: i.ordered_quantity as u32,
            tax_rate: i.tax_rate as u32
        }
    } ).collect();

    Ok(res)

}

pub async fn undo_reservation(uuid: String ) -> Result<(),CheckoutError>
{

    info!("Undoing reservation {}", uuid);

    let uuid = sqlx::types::Uuid::from_str(uuid.as_ref())?;
    let mut tx = crate::postgres::POOL.get().unwrap().begin().await?;

    struct T {
        product_id: i32,
        quantity: i32
    }


    {
        let reservations = query_as!(T,"DELETE FROM product_reservations WHERE (reservation_id=$1) RETURNING product_id,quantity",uuid)
        .fetch_all(&mut *tx).await?;


        for r in reservations
        {
            query!("UPDATE products  SET  quantity=(quantity+$1) WHERE (quantity IS NOT NULL and id=$2)",r.quantity,r.product_id )
            .execute(&mut *tx)
            .await?;
        }

    }

    query!("DELETE FROM pending_orders WHERE id=$1",uuid)
    .execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(())

}   

pub async fn commit_reservation(uuid: String ) -> Result<(),CheckoutError>
{

    info!("Commiting reservation {}", uuid);

    let uuid = sqlx::types::Uuid::from_str(uuid.as_ref())?;
    let mut tx = crate::postgres::POOL.get().unwrap().begin().await?;

    query!("DELETE FROM product_reservations WHERE (reservation_id=$1)",uuid)
    .execute (&mut *tx).await?;

    query!("DELETE FROM pending_orders WHERE id=$1",uuid)
    .execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(())

}   