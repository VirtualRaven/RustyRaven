
mod postgres;

pub mod category;
pub mod product;
pub mod checkout;
pub use product::*;
pub use postgres::image as image;
pub use sqlx::Error as Error;
use tracing::{info,error};

pub struct DbSettings {
    db_user: String,
    db_address: String,
    db_name: String,
    db_password: String
}

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
            info!("Removing stale reservations");
            match checkout::undo_old_reservations().await
            {
                Err(e) => error!("Periodic reservation cleanup failed with error {}",e),
                _ => ()
            }
            interval.tick().await;
        }
    });

    return true

}