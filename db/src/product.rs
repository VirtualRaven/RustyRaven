use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sjf_api::{category, product::{GetPreviewsRequest, GetPreviewsResp, Preview,Product as ApiProduct,GetProductRequest,GetProductResponse}};
use sqlx::{database, postgres::{PgHasArrayType, PgPoolOptions},query, query_file, query_file_as, Pool, Postgres};
use crate::postgres::POOL;


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
    pub images: Vec<u32>,
    pub category: u32,
}

pub async fn get_products(category: u32) -> Result<Vec<Product>,sqlx::Error>
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
        pub image_ids: Option<Vec<i32>>,
        pub category: i32
    }

    impl From<ProductT> for Product {
        fn from(p: ProductT) -> Self {
            Product { id: p.id, name: p.name, price: p.price, description: p.description, quantity: p.quantity, created: p.created, updated: p.updated, product_tag: p.product_tag, tax_rate: p.tax_rate as u32, category: p.category as u32,  images: p.image_ids.unwrap_or_default().into_iter().map(|x| x as u32).collect() }
        }
    }
   

    let res : Vec<_> =query_file_as!(ProductT,"sql/all_products.sql", category as i32)
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
    product.category as i32,
    product.tax_rate as i32,
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
    crate::image::update_image_view_later();


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

    crate::image::update_image_view_later();

    Ok(())
}



#[derive(sqlx::Type)]
#[sqlx(type_name = "image_variant")]
struct ImageVariant {
    width: i32,
    height: i32,
    variant: i32,
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "image_info_type")]
struct ImageInfo {
    id: i32,
    avg_color: String,
    variants: Vec<ImageVariant>
}

pub async fn get_previews(req: GetPreviewsRequest) -> Result<GetPreviewsResp,sqlx::Error>
{   
    let categories = crate::category::get_child_categories(req.category, req.recursive, POOL.get().unwrap()).await?;
    

    struct  T {
        id: Option<i32>,
        price: Option<i32>,
        name: Option<String>,
        images: Option<Vec<ImageInfo>>,
        names: Option<Vec<String>>,
    };

    let q = query_file_as!(T,"sql/get_previews.sql",&categories,req.limit as i32 )
    .fetch_all(POOL.get().unwrap())
    .await?;



    let result =  q.into_iter().map(|t|  {
        Preview 
        {
            id: t.id.unwrap() as u32,
            name: t.name.unwrap(),
            price: t.price.unwrap() as u32,
            images: t.images.unwrap_or_default().into_iter().map( |i|
            {
                sjf_api::product::Image {
                    color: i.avg_color,
                    sizes: i.variants.into_iter().map(|v| {
                        sjf_api::product::ImageVariant {
                            width: v.width as u32,
                            height: v.height as u32,
                            url: format!("/images/{}/{}",i.id,v.variant)
                        }
                    } ).collect()
                }
            }).collect(),
            category_name: t.names.unwrap()
        }
    } ).collect();

    Ok(
        GetPreviewsResp { previews: result }
    )

}

pub async fn get_product(req: GetProductRequest) -> Result<GetProductResponse,sqlx::Error>
{   
    
    struct  T {
        id: Option<i32>,
        price: Option<i32>,
        description: Option<String>,
        name: Option<String>,
        images: Option<Vec<ImageInfo>>,
        names: Option<Vec<String>>,
    };

    let t = query_file_as!(T,"sql/get_product.sql", req.product_id as i32 )
    .fetch_one(POOL.get().unwrap())
    .await?;



    let result = 
        ApiProduct 
        {
            id: t.id.unwrap() as u32,
            name: t.name.unwrap(),
            description: t.description.unwrap(),
            price: t.price.unwrap() as u32,
            images: t.images.unwrap_or_default().into_iter().map( |i|
            {
                sjf_api::product::Image {
                    color: i.avg_color,
                    sizes: i.variants.into_iter().map(|v| {
                        sjf_api::product::ImageVariant {
                            width: v.width as u32,
                            height: v.height as u32,
                            url: format!("/images/{}/{}",i.id,v.variant)
                        }
                    } ).collect()
                }
            }).collect(),
            category_name: t.names.unwrap()
        };

    Ok(
        result 
    )

}