use sqlx::{pool::PoolConnection, MySql};

use crate::{helpers::get_context, ServerContext};

pub mod snare;

pub async fn get_conn<T: From<sqlx::Error>>() -> Result<PoolConnection<MySql>, T> {
    let svr_ctx = get_context(|core_ctx: &ServerContext| core_ctx.clone());
    let conn = svr_ctx
        .db_pool
        .expect("Database isn't enabled in cali config, why are you asking me for a connection?")
        .acquire()
        .await?;
    Ok(conn)
}
