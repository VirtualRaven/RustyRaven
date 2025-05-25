
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

    let settings = DbSettings {

        db_user: { std::env::var("POSTGRES_USER")
            .expect("POSTGRES_USER environment variable has to be set. ")},
        db_address: { std::env::var("POSTGRES_ADDRESS")
            .expect("POSTGRES_ADDRESS environment variable has to be set. ")},
        db_name: { std::env::var("POSTGRES_DB_NAME")
            .expect("POSTGRES_DB_NAME environment variable has to be set. ")},
        db_password: {std::env::var("POSTGRES_PASSWORD")
            .expect("POSTGRES_PASSWORD environment variable has to be set. ")}
    };
    if let Err(e) = postgres::init(&settings).await
    {
        log::error!("Failed to initialize DB connection {:#?}",e);
        return false;
    }
    return true

}