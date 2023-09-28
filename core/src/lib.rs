use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

pub mod config;
pub mod helpers;
pub mod logging;
pub mod middleware;
pub mod protos;
pub mod scaffolding;
pub mod store;

#[derive(Debug, Clone)]
pub struct ServerContext<T> {
    pub db_pool: sqlx::MySqlPool,
    pub config: T,
}

pub type MapKey = Arc<dyn Any + Send + Sync>;

tokio::task_local! {
    pub static SERVER_CONTEXT: Arc<HashMap<TypeId,MapKey>>;
}
