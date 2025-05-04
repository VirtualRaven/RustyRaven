
#[tokio::main]
async fn main() {
    let args = Args::parse();
    pretty_env_logger::init();
    if let Err(e) = postgres::init(&args).await
    {
        log::error!("Failed to initialize DB connection {:#?}",e);
        return;
    }
    log::info!("Hello world!");
}