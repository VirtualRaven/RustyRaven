mod postgres;

pub mod auth;
pub mod category;
pub mod checkout;
pub mod product;
pub use postgres::image;
pub use product::*;
pub use sqlx::Error;
use sqlx::{query, query_file};
use tracing::{error, info};

async fn update_gauges() -> Result<(), sqlx::Error> {
    query!("ANALYZE products,users,images,image_variants,pending_orders,product_categories,product_images, product_reservations")
    .execute(crate::postgres::POOL.get().unwrap()).await?;
    let rows = query_file!("sql/size_estimates.sql")
        .fetch_all(crate::postgres::POOL.get().unwrap())
        .await?;

    for row in rows {
        let lables = [("table", row.relname)];
        metrics::gauge!("postgres_table_size_estimate", &lables).set(row.estimate)
    }

    Ok(())
}

pub async fn init() -> bool {
    if let Err(e) = postgres::init().await {
        tracing::error!("Failed to initialize DB connection {:#?}", e);
        return false;
    }

    tokio::task::spawn(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10 * 60));
        loop {
            interval.tick().await;
            info!("Removing stale reservations");
            match checkout::undo_old_reservations().await {
                Err(e) => error!("Periodic reservation cleanup failed with error {}", e),
                _ => (),
            }
        }
    });

    tokio::task::spawn(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10 * 60));
        loop {
            interval.tick().await;
            match update_gauges().await {
                Err(e) => error!("Periodic gauge check failed with error {}", e),
                _ => (),
            }
        }
    });

    return true;
}
