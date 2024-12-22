use async_trait::async_trait;
use std::future::Future;
use tokio::task::JoinHandle;

#[async_trait]
pub trait Dispatcher: Send + Sync {
    async fn schedule<F>(&self, f: F) -> JoinHandle<()>
    where
        F: Future<Output = ()> + Send + 'static;
}

pub struct ThreadPoolDispatcher {
    pool: tokio::runtime::Runtime,
}

impl ThreadPoolDispatcher {
    pub fn new(threads: usize) -> Self {
        let pool = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(threads)
            .enable_all()
            .build()
            .unwrap();
            
        Self { pool }
    }
}

#[async_trait]
impl Dispatcher for ThreadPoolDispatcher {
    async fn schedule<F>(&self, f: F) -> JoinHandle<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.pool.spawn(f)
    }
} 