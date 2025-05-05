use std::{borrow::Cow, collections::HashMap, sync::Mutex};

use wasmtime::CacheStore;

#[derive(Debug)]
pub struct Cache;

static CACHE: Mutex<Option<HashMap<Vec<u8>, Vec<u8>>>> = Mutex::new(None);

impl CacheStore for Cache {
    fn get(&self, key: &[u8]) -> Option<Cow<[u8]>> {
        let mut cache = CACHE.lock().unwrap();
        let cache = cache.get_or_insert_with(HashMap::new);
        cache.get(key).map(|s| s.to_vec().into())
    }

    fn insert(&self, key: &[u8], value: Vec<u8>) -> bool {
        let mut cache = CACHE.lock().unwrap();
        let cache = cache.get_or_insert_with(HashMap::new);
        cache.insert(key.to_vec(), value);
        true
    }
}
