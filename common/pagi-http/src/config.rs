use std::net::SocketAddr;

pub fn bind_addr(default: SocketAddr) -> SocketAddr {
    let Ok(val) = std::env::var("BIND_ADDR") else {
        return default;
    };
    val.parse().unwrap_or(default)
}

