use std::{sync::Arc, collections::HashMap, any::{TypeId, Any}};

pub mod helpers;
pub mod store;
pub mod logging;
pub mod protos;
pub mod scaffolding;
pub mod middleware;

pub struct ServerContext {
    pub db_pool: sqlx::MySqlPool,
}

tokio::task_local! {
    pub static SERVER_CONTEXT: Arc<HashMap<TypeId, Arc<dyn Any>>>;
}
