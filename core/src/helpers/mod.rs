use std::str::FromStr;

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
