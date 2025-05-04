
mod postgres;
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// DB username (password passed via POSTGRES_PASSWORD environment variable)
    #[arg(long)]
    db_user: String,
    // Address to postgreSQL server
    #[arg(long)]
    db_address: String,
    // Database name
    #[arg( long)]
    db_name: String
}

pub async fn init() -> bool
{
    let args = Args::parse();
    if let Err(e) = postgres::init(&args).await
    {
        log::error!("Failed to initialize DB connection {:#?}",e);
        return false;
    }
    return true

}