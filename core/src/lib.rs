use std::sync::Arc;

pub mod helpers;
pub mod logging;
pub mod protos;
pub mod scaffolding;

pub struct ServerContext {
    pub db_pool: sqlx::MySqlPool,
}

tokio::task_local! {
    pub static SERVER_CONTEXT: Arc<ServerContext>;
}
