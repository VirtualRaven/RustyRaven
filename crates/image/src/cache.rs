use std::{collections::{BTreeMap, VecDeque}, sync::Arc};
use tracing::info;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use chrono::{naive, Local, NaiveDateTime, Utc};
use lazy_static::lazy_static;

use crate::ImageId;

struct CacheEntry {
    data: Arc<Vec<u8>>,
    timestamp: NaiveDateTime
}
struct CacheContent {
    cache_size: u64,
    images: BTreeMap<ImageId, CacheEntry>
}

impl CacheContent {
    fn new() -> Self {
        CacheContent  { cache_size: 0, images: BTreeMap::new() }
    }
}

struct ImageCache {
    lock: RwLock<CacheContent>
}


lazy_static! {
    static ref CACHE: ImageCache = ImageCache::new();

}

impl ImageCache {

    const MAX_SIZE: u64    = 256*1024*1024;
    const TARGET_SIZE: u64 = 128*1024*1024;

    fn new() -> Self {
        ImageCache { lock: RwLock::new(CacheContent::new() ) }
    
    }
}

pub async fn add_image(id: ImageId, data: Vec<u8> )
{
    info!("Adding {:?} to cache",id);
    assert!((data.len() as u64) < ImageCache::TARGET_SIZE);
    let mut cache = CACHE.lock.write().await;
    cache.cache_size += data.len() as u64;
    cache.images.insert(id, 
        CacheEntry { data: Arc::new(data), timestamp: Utc::now().naive_utc()  }
    );
    enforce_size(&mut cache);
}

async fn update_acces_time( id: ImageId, access_time: NaiveDateTime)
{
    let mut cache = CACHE.lock.write().await;
    match cache.images.get_mut(&id)
    {
        Some(e) => {
            e.timestamp = access_time;
        }
        None => ()
    } 
} 

pub async fn get_image(id: ImageId) -> Option<Arc<Vec<u8>>>
{
    let cache = CACHE.lock.read().await;

    match cache.images.get(&id)
    {
        Some(e) => {
            let res = e.data.clone();

            let now = Utc::now().naive_utc();
            if  now - e.timestamp > chrono::Duration::minutes(5) 
            {
                tokio::spawn(
                    async move {
                        update_acces_time(id,now).await;
                    }
                );
            }
            Some(res)
        },
        None => None
    }

}

fn enforce_size<'a>(cache: &mut RwLockWriteGuard<'a,CacheContent>)
{
    if cache.cache_size > ImageCache::MAX_SIZE
    {
        info!("Image cache limit hit");
        let mut tmp: Vec<(NaiveDateTime,ImageId)> = cache.images.iter().map(|(k,v)|  (v.timestamp.clone(), (*k).clone()) ).collect();
        tmp.sort_by(|(a,_),(b,_)| a.cmp(b));
        let mut oldest = tmp.into_iter().map(|(_,x)| x);

        while cache.cache_size > ImageCache::TARGET_SIZE
        {
            let oldest_id = oldest.next().unwrap();  //We know any element of the cache is smaller than the target size
            remove_image(cache, &oldest_id );
        } 
    }

}

fn  remove_image<'a>(cache: &mut RwLockWriteGuard<'a,CacheContent>,  id: &ImageId)
{
    match cache.images.remove(id)
    {
        Some(e) => {
            cache.cache_size -= e.data.len() as u64
        },
        None => {}
    }
    
}

