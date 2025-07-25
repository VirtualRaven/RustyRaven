use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sjf_api::category;
use sqlx::{database, postgres::{PgHasArrayType, PgPoolOptions},query, query_file, query_file_as, Pool, Postgres};
use once_cell::sync::OnceCell;

pub (crate) static POOL: OnceCell<Pool<Postgres>> = OnceCell::new();

pub async fn init() -> Result<(),sqlx::Error>
{

    let url = std::env::var("DATABASE_URL").unwrap_or_else( |_| {
        let user = std::env::var("POSTGRES_USER")
        .expect("POSTGRES_USER environment variable has to be set. ");
        let address =  std::env::var("POSTGRES_ADDRESS")
        .expect("POSTGRES_ADDRESS environment variable has to be set. ");
        let database =  std::env::var("POSTGRES_DB_NAME")
        .expect("POSTGRES_DB_NAME environment variable has to be set. ");
        let password =  std::env::var("POSTGRES_PASSWORD")
        .expect("POSTGRES_PASSWORD environment variable has to be set. ");
        format!("postgres://{user}:{password}@{address}/{database}")
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url).await?;

    {
        let _ = query_file!("sql/type_definitions/0-image_variant.sql").execute(&pool).await;
        let _ = query_file!("sql/type_definitions/1-image_info_type.sql").execute(&pool).await;
    }


    {
        let mut tx = pool.begin().await?;
        query_file!("sql/table_definitions/0-product_categories.sql").execute(&mut *tx).await?;
        query_file!("sql/table_definitions/1-product_categories_hierarchy.sql").execute(&mut *tx).await?;
        query_file!("sql/table_definitions/2-images.sql").execute(&mut *tx).await?;
        query_file!("sql/table_definitions/3-image_variants.sql").execute(&mut *tx).await?;
        query_file!("sql/table_definitions/4-products.sql").execute(&mut *tx).await?;
        query_file!("sql/table_definitions/5-product_images.sql").execute(&mut *tx).await?;
        query_file!("sql/table_definitions/6-product_association_index.sql").execute(&mut *tx).await?;
        tx.commit().await?;
    }

    image::update_image_view(&pool, true).await?;
    crate::category::update_paths_view(&pool, true).await?;
    POOL.set(pool).unwrap();
    Ok(())
}




pub mod image {

    use std::collections::BTreeMap;

    use tracing::error;
    use sqlx::{query, query_as, Executor};

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

pub async fn update_image_view<'c,E>( e : E, create: bool  ) -> Result<(),sqlx::Error> 
where E: Copy + Executor<'c, Database = Postgres>,
{
    if (create)
    {
        query_file!("sql/materialized_image_view.sql").execute(e).await?;
        query!("CREATE UNIQUE INDEX IF NOT EXISTS product_image_info_index  on product_image_info (product_id)").execute(e).await?;
    }

    query!("REFRESH MATERIALIZED VIEW CONCURRENTLY product_image_info").execute(e).await?;
    Ok(())
}

pub fn update_image_view_later() 
{
    tokio::spawn(async {
        let res = update_image_view(POOL.get().unwrap(), false).await;
        if let Err(e) = res
        {
            error!("Image view update failed! {:#?}",e);
        }

    });
}

}
