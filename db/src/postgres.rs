use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{database, postgres::{PgHasArrayType, PgPoolOptions},query, query_file, query_file_as, Pool, Postgres};
use once_cell::sync::OnceCell;

static POOL: OnceCell<Pool<Postgres>> = OnceCell::new();

pub async fn init(args: &crate::DbSettings) -> Result<(),sqlx::Error>
{

    let url = {
        let user = &args.db_user;
        let database = &args.db_name;
        let address = &args.db_address;
        let password = &args.db_password;

        format!("postgres://{user}:{password}@{address}/{database}")
    };

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url).await?;

    POOL.set(pool).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "product_tag", rename_all = "lowercase")] 
pub enum ProductTag {
    Clothing,
    Lamp
}


#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct Product {

    pub id: i32,
    pub name: String, //VARCHAR(100)
    pub price: i32,
    pub description: String,
    pub quantity: Option<i32>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub product_tag: Vec<ProductTag>,
    pub tax_rate: u32,
    pub images: Vec<u32>
}

pub async fn get_products() -> Result<Vec<Product>,sqlx::Error>
{
    #[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
    pub struct ProductT {

        pub id: i32,
        pub name: String, //VARCHAR(100)
        pub price: i32,
        pub description: String,
        pub quantity: Option<i32>,
        pub created: DateTime<Utc>,
        pub updated: DateTime<Utc>,
        pub product_tag: Vec<ProductTag>,
        pub tax_rate: i32,
        pub image_ids: Option<Vec<i32>>
    }

    impl From<ProductT> for Product {
        fn from(p: ProductT) -> Self {
            Product { id: p.id, name: p.name, price: p.price, description: p.description, quantity: p.quantity, created: p.created, updated: p.updated, product_tag: p.product_tag, tax_rate: p.tax_rate as u32, images: p.image_ids.unwrap_or_default().into_iter().map(|x| x as u32).collect() }
        }
    }
   

    let res : Vec<_> =query_file_as!(ProductT,"sql/all_products.sql")
        .fetch_all(POOL.get().unwrap())
        .await?
        .into_iter()
        .map(|x| x.into())
        .collect();
    Ok(res)


}

pub async fn create_product(product: Product ) -> Result<i32, sqlx::Error>
{
    let mut tx = POOL.get().unwrap().begin().await?;

    let query = query_file!("sql/create_product.sql",
    product.name,
    product.price,
    product.description,
    product.quantity,
    product.product_tag as _,
    )
    .fetch_one(&mut *tx)
    .await?;

    for image in product.images
    {
        query!("INSERT INTO product_images (product_id,image_id) VALUES ($1,$2)",query.id,image as i32)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;


    Ok(query.id)
}
pub async fn update_product(product: Product ) -> Result<(), sqlx::Error>
{
    let mut tx = POOL.get().unwrap().begin().await?;



    let query = query_file!("sql/update_product.sql",
    product.name,
    product.price,
    product.description,
    product.quantity,
    product.product_tag as _,
    product.id)
    .execute (&mut *tx)
    .await?;

    query!("DELETE from product_images where product_id=$1",product.id)
    .execute(&mut *tx)
    .await?;
    
    for image in product.images
    {
        query!("INSERT INTO product_images (product_id,image_id) VALUES ($1,$2)",product.id,image as i32)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub mod image {

    use std::collections::BTreeMap;

    use sqlx::{query, query_as};

    use super::*;
pub struct ImageInsertVariant {
    pub height: i32,
    pub width: i32,
}

pub struct ImageInsertRequest{
    pub avg_color: String,
    pub variants: [ImageInsertVariant; 4]
}

pub struct InsertedImage {
    pub image_id: i32,
    pub variant_id: i32
}

pub async fn insert_image(image: ImageInsertRequest) -> Result<Vec<InsertedImage>, sqlx::Error>
{
    query_file_as!(InsertedImage,"sql/images/create_image.sql",
        image.avg_color,
        image.variants[0].width,
        image.variants[0].height,
        image.variants[1].width,
        image.variants[1].height,
        image.variants[2].width,
        image.variants[2].height,
        image.variants[3].width,
        image.variants[3].height,
    ).fetch_all(POOL.get().unwrap()).await

}

pub async fn get_product_images(product_id: u32) -> Result<BTreeMap<u32,Vec<u32>>,sqlx::Error>
{

    let res = query_file!("sql/images/product_images.sql",product_id as i32)
    .fetch_all(POOL.get().unwrap())
    .await?
    .into_iter()
    .map(|x| (x.image_id as u32,x.variant_ids.unwrap().into_iter().map(|x| x as u32).collect() ))
    .collect();

    Ok(res)
}

pub async fn get_image_variants(image_id: u32) -> Result<Vec<u32>,sqlx::Error>
{

    let res = query!("SELECT variant_id, (width * height) AS size FROM image_variants WHERE image_id=$1 ORDER BY size ASC",image_id as i32 )
    .fetch_all(POOL.get().unwrap())
    .await?
    .into_iter()
    .map(|x| x.variant_id as u32)
    .collect();

    Ok(res)
}


}