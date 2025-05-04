use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{database, postgres::{PgHasArrayType, PgPoolOptions}, query_file, query_file_as, Pool, Postgres};
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
    pub images: Vec<String>,
}

pub async fn get_products() -> Result<Vec<Product>,sqlx::Error>
{
    query_file_as!(Product,"sql/all_products.sql").fetch_all(POOL.get().unwrap()).await
}

pub async fn create_product(product: Product) -> Result<i32, sqlx::Error>
{
    #[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
    struct Response {
        id: i32
    }
    let query = query_file_as!(Response,"sql/create_product.sql",
    product.name,
    product.price,
    product.description,
    product.quantity,
    product.product_tag as _,
    &product.images)
    .fetch_one(POOL.get().unwrap())
    .await?;
    Ok(query.id)
}
pub async fn update_product(product: Product) -> Result<(), sqlx::Error>
{
    #[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
    struct Response {
        id: i32
    }
    let query = query_file_as!(Response,"sql/update_product.sql",
    product.name,
    product.price,
    product.description,
    product.quantity,
    product.product_tag as _,
    &product.images,
    product.id)
    .execute (POOL.get().unwrap())
    .await?;
    Ok(())
}

//async fn create_tables()  -> Result<(),sqlx::Error>
//{
//    //query_file!("sql/product_tag.sql").execute(POOL.get().unwrap()).await?;
//    //Ok(())
//}