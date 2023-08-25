use std::str::FromStr;

use crate::SERVER_CONTEXT;

pub fn split_host_and_port(addr: &str) -> (&str, u16) {
    let parts = addr.split(':').collect::<Vec<_>>();
    if parts.len() >= 2 {
        match u16::from_str(parts[1]) {
            Ok(port) => (parts[0], port),
            Err(_) => (parts[0], 0),
        }
    } else {
        (addr, 0)
    }
}

pub fn get_context<R, T: 'static>(thunk: impl FnOnce(&T) -> R) -> R {
    SERVER_CONTEXT.with(
        |ctx| match ctx.get(&std::any::TypeId::of::<T>()) {
            Some(svr_ctx) => thunk(
                svr_ctx
                    .downcast_ref::<T>()
                    .expect("Guaranteed by HashMap structure"),
            ),
            None => panic!("Guaranteed by middleware"),
        },
    )
}
