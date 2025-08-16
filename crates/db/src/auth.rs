use sqlx::{ query, query_file, types::Uuid, Postgres, Transaction };
use crate::postgres::POOL;

pub async fn add(id: Uuid, name: String, keyid: Vec<u8>, passkey: Vec<u8>) -> Result<(), sqlx::Error >
{

    let mut tx = POOL.get().unwrap().begin().await?;
    query_file!("sql/create_user.sql",id,name).execute(&mut *tx).await?;

    query!("INSERT INTO user_passkeys (user_id,keyid,passkey) VALUES ($1,$2,$3)",id,keyid,passkey).execute(&mut *tx).await?;
    tx.commit().await

}


pub async fn lookup_name(name: &str) -> Result<Option<Uuid>,sqlx::Error>
{
    let res = query!("SELECT id FROM users WHERE name=$1",name). fetch_optional(POOL.get().unwrap()).await?;
    Ok(res.map(|r| r.id))
}
pub async fn lookup_id(id: &Uuid) -> Result<Option<String>,sqlx::Error>
{
    let res = query!("SELECT name FROM users WHERE id=$1",id). fetch_optional(POOL.get().unwrap()).await?;
    Ok(res.map(|r| r.name.unwrap()))
}

pub async fn get_keys(id: &Uuid) -> Result<Vec<Vec<u8>>,sqlx::Error>
{
    Ok(query!("SELECT passkey FROM user_passkeys WHERE user_id=$1", id).fetch_all(POOL.get().unwrap()).await?.into_iter().map(|r| r.passkey.unwrap()).collect())
}

pub async fn begin_passkey_update(id: Uuid, keyid: Vec<u8>) -> Result<(Transaction<'static,Postgres>, Vec<u8>), sqlx::Error >
{

    let mut tx = POOL.get().unwrap().begin().await?;
    let k = query!("SELECT passkey from user_passkeys where (keyid=$1 and user_id=$2)",keyid,id).fetch_one(&mut *tx).await?;

    Ok((tx,k.passkey.unwrap()))
}

pub async fn complete_passkey_update(mut tx: Transaction<'static,Postgres>, id: Uuid, keyid: Vec<u8>, passkey: Vec<u8>) -> Result<(), sqlx::Error >
{

    query!("UPDATE user_passkeys SET  passkey=$1 where (keyid=$2 and user_id=$3)",passkey,keyid,id).execute(&mut *tx).await?;
    tx.commit().await

}