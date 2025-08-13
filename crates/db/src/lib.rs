
mod postgres;

pub mod category;
pub mod product;
pub mod checkout;
pub use product::*;
pub use postgres::image as image;
pub use sqlx::Error as Error;
use tracing::{info,error};

pub async fn init() -> bool
{

    if let Err(e) = postgres::init().await
    {
        tracing::error!("Failed to initialize DB connection {:#?}",e);
        return false;
    }
    
    tokio::task::spawn(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10*60));
        loop {
            interval.tick().await;
            info!("Removing stale reservations");
            match checkout::undo_old_reservations().await
            {
                Err(e) => error!("Periodic reservation cleanup failed with error {}",e),
                _ => ()
            }
        }
    });

    return true

}