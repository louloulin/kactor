use std::any::{Any, TypeId};
use dashmap::DashMap;
use std::sync::Arc;

pub struct ActorSystemExtensions {
    extensions: DashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ActorSystemExtensions {
    pub fn new() -> Self {
        Self {
            extensions: DashMap::new(),
        }
    }

    pub fn register<T: 'static + Send + Sync>(&self, extension: T) {
        self.extensions.insert(TypeId::of::<T>(), Box::new(extension));
    }

    pub fn get<T: 'static>(&self) -> Option<Arc<T>> {
        self.extensions
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<Arc<T>>().cloned())
    }

    pub fn remove<T: 'static>(&self) -> Option<Box<dyn Any + Send + Sync>> {
        self.extensions.remove(&TypeId::of::<T>()).map(|pair| pair.1)
    }
}

impl Default for ActorSystemExtensions {
    fn default() -> Self {
        Self::new()
    }
}