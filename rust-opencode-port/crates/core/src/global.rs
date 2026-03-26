use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct GlobalState {
    data: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn set<T: Send + Sync + 'static>(&self, value: T) {
        let type_id = TypeId::of::<T>();
        let mut data = self.data.write().await;
        data.insert(type_id, Box::new(value));
    }

    pub async fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();
        let data = self.data.read().await;
        data.get(&type_id)
            .and_then(|v| v.downcast_ref::<T>())
            .map(|v| Arc::new(unsafe { std::ptr::read(v as *const T) }))
    }

    pub async fn has<T: Send + Sync + 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        let data = self.data.read().await;
        data.contains_key(&type_id)
    }

    pub async fn remove<T: Send + Sync + 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        let mut data = self.data.write().await;
        data.remove(&type_id).is_some()
    }
}

impl Default for GlobalState {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for GlobalState {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}
