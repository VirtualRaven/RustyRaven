use std::collections::HashMap;

use log::{error, info};
use sjf_api::category::{CreateReq,CreateRsp, GetChildrenRsp};
use sqlx::{query, query_as,query_file, Executor, Postgres};
use crate::postgres::POOL;

pub async fn create(req: CreateReq ) -> Result<CreateRsp,sqlx::Error>
{
    let mut tx = POOL.get().unwrap().begin().await?;

    let res = query!("INSERT INTO product_categories (name) VALUES ($1) RETURNING id",req.name )
    .fetch_one(&mut *tx)
    .await?;


    let depth = {
        if let Some(parent) = req.parent
        {
            query!("SELECT depth FROM product_categories_hierarchy where (ancestor=$1 and descendant=$1)",parent as i32)
            .fetch_one(&mut *tx)
            .await.map(|r| r.depth+1)
        }
        else {
            Ok(0)
        }
    }?;

    query!("INSERT INTO product_categories_hierarchy (ancestor,descendant,depth) VALUES($1,$1,$2)", res.id,depth)
    .execute(&mut *tx)
    .await?;

    if depth > 0 
    {
        query!("INSERT INTO product_categories_hierarchy (ancestor,descendant,depth) SELECT ancestor,$1,$2 FROM product_categories_hierarchy where descendant=$3", res.id as i32,depth as i32,req.parent.unwrap() as i32)
        .execute(&mut *tx)
        .await?;
    }


    tx.commit().await?;
    update_paths_view_later();
    Ok(CreateRsp { id: res.id as u32, depth: depth as u32})
}


pub async fn get_children(id: Option<u32> ) -> Result<GetChildrenRsp,sqlx::Error>
{
    struct Res {
        id: i32,
        name: String
    }

    let rsp =  match id 
    {
        Some(id) => 
            query_as!(Res,"WITH D AS (SELECT depth FROM product_categories_hierarchy where ancestor=descendant and ancestor=$1 )  SELECT  pc.id,pc.name FROM product_categories pc JOIN product_categories_hierarchy ph ON (pc.id = ph.descendant) WHERE ph.ancestor = $1 AND ph.depth = ((select depth from D)+1) ",id as i32)
            .fetch_all(POOL.get().unwrap())
            .await,
        None => 
            query_as!(Res,"SELECT pc.id,pc.name  FROM product_categories pc JOIN product_categories_hierarchy ph ON (pc.id = ph.descendant) WHERE ph.ancestor = ph.descendant AND ph.depth = 0")
            .fetch_all(POOL.get().unwrap())
            .await
    }?;

    Ok(
        GetChildrenRsp {
            children: rsp.into_iter().map(|r| (r.id as u32,r.name)).collect()
        }
    )

}

pub async fn get_child_categories<'c,E>(root: Option<u32>, recursive: bool, e : E  ) -> Result<Vec<i32>,sqlx::Error> 
where E: Executor<'c, Database = Postgres>,
{
    struct T {
        descendant: i32
    };

    info!("Banan");

    match (root,recursive)
    {
        (None,true) => 
            query_as!(T,"SELECT descendant FROM product_categories_hierarchy ORDER BY depth ASC")
            .fetch_all(e)
            .await,
        (None,false) => 
            query_as!(T,"SELECT descendant FROM product_categories_hierarchy WHERE depth=0 ORDER BY depth ASC")
            .fetch_all(e)
            .await,
        (Some(parent),false) => 
            query_as!(T,"WITH D AS (SELECT depth from product_categories_hierarchy WHERE (descendant=ancestor AND ancestor=$1)) SELECT descendant FROM product_categories_hierarchy WHERE (ancestor=$1 AND depth = (SELECT depth from D) ) ",parent as i32)
            .fetch_all(e)
            .await,
        (Some(parent),true) => 
            query_as!(T,"WITH D AS (SELECT depth from product_categories_hierarchy WHERE (descendant=ancestor AND ancestor=$1)) SELECT descendant FROM product_categories_hierarchy WHERE (ancestor=$1 AND depth >= (SELECT depth from D) ) ",parent as i32)
            .fetch_all(e)
            .await
    }
    .map(|vs| vs.into_iter().map(|v| v.descendant).collect() )

}

pub async fn update_name(id: u32, name: String) -> Result<(),sqlx::Error>
{
    query!("UPDATE product_categories SET name=$1 where id=$2 RETURNING ID",name,id as i32)
    .fetch_one(POOL.get().unwrap())
    .await?;
    update_paths_view_later();

    Ok(())
}


pub(crate) async fn update_paths_view<'c,E>( e : E, create: bool  ) -> Result<(),sqlx::Error> 
where E: Copy + Executor<'c, Database = Postgres>,
{
    if (create)
    {
        query_file!("sql/materialized_product_paths.sql").execute(e).await?;
        query!("CREATE UNIQUE INDEX IF NOT EXISTS product_paths_index  on product_paths (id)").execute(e).await?;
    }

    query!("REFRESH MATERIALIZED VIEW CONCURRENTLY product_paths").execute(e).await?;
    Ok(())
}

pub(crate) fn update_paths_view_later() 
{
    tokio::spawn(async {
        let res = update_paths_view(POOL.get().unwrap(), false).await;
        if let Err(e) = res
        {
            error!("Path view update failed! {:#?}",e);
        }

    });
}

pub async fn get_paths() -> Result<HashMap<String,u32> , sqlx::Error >
{
    let res = query!("SELECT id, array_to_string(names,'/') as path FROM product_paths")
    .fetch_all(POOL.get().unwrap())
    .await?;

    Ok(res.into_iter().map(|r| (r.path.unwrap(),r.id.unwrap() as u32)  ).collect())
}