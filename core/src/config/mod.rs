use std::sync::Arc;

use crate::{helpers::get_context, ServerContext};

pub fn get_config<C: 'static>() -> Arc<C> {
    let svr_ctx = get_context(|core_ctx: &ServerContext<Arc<C>>| core_ctx.clone());

    svr_ctx.config
}
