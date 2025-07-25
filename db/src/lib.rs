
mod postgres;

pub mod category;
pub mod product;
pub use product::*;
pub use postgres::image as image;
pub use sqlx::Error as Error;

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
    return true

}