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
pub mod store;

#[derive(Debug, Clone)]
pub struct ServerContext {
    pub db_pool: Option<sqlx::MySqlPool>,
}

pub type MapKey = Arc<dyn Any + Send + Sync>;

tokio::task_local! {
    pub static SERVER_CONTEXT: Arc<HashMap<TypeId,MapKey>>;
}
