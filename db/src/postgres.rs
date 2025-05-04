use sqlx::{database, postgres::PgPoolOptions, query_file, Pool, Postgres};
use once_cell::sync::OnceCell;

static POOL: OnceCell<Pool<Postgres>> = OnceCell::new();

pub async fn init(args: &crate::Args) -> Result<(),sqlx::Error>
{
    let password = std::env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD environment variable has to be set!");

    let url = {
        let user = &args.db_user;
        let database = &args.db_name;
        let address = &args.db_address;
        format!("postgres://{user}:{password}@{address}/{database}")
    };

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url).await?;

    POOL.set(pool).unwrap();
    Ok(())
}


//async fn create_tables()  -> Result<(),sqlx::Error>
//{
//    //query_file!("sql/product_tag.sql").execute(POOL.get().unwrap()).await?;
//    //Ok(())
//}