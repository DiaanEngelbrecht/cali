use std::sync::Arc;

use sqlx::{pool::PoolConnection, MySql};

use crate::{helpers::get_context, ServerContext};

pub mod snare;

pub async fn get_conn<T: From<sqlx::Error>, C: 'static>() -> Result<PoolConnection<MySql>, T> {
    let svr_ctx = get_context(|core_ctx: &ServerContext<Arc<C>>| core_ctx.clone());
    let conn = svr_ctx.db_pool.acquire().await?;
    Ok(conn)
}
