use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct KeyValueStore {
    store: Arc<Mutex<HashMap<String, String>>>
}

impl KeyValueStore {
    pub fn new() -> Self {
        KeyValueStore {
            store: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub async fn set(&self, key: String, value: String) {
        let mut store = self.store.lock().await;
        store.insert(key, value);
    }

    pub async fn get(&self, key: String) -> Option<String> {
        let store = self.store.lock().await;
        store.get(&key).cloned()
    }

    pub async fn delete(&self, key: &str) {
        let mut store = self.store.lock().await;
        store.remove(key);
    }
}