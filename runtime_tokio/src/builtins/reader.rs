use std::net::SocketAddr;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[repr(C)]
pub enum Reader {
    Stdin,
    File { path: PathBuf, watch: bool },
    Http { addr: SocketAddr },
    Tcp { addr: SocketAddr },
    Kafka { addr: SocketAddr, topic: String },
}

impl Reader {
    pub fn stdin() -> Self {
        Self::Stdin
    }
    pub fn file(path: PathBuf, watch: bool) -> Self {
        Self::File { path, watch }
    }
    pub fn http(addr: SocketAddr) -> Self {
        Self::Http { addr }
    }
    pub fn tcp(addr: SocketAddr) -> Self {
        Self::Tcp { addr }
    }
    pub fn kafka(addr: SocketAddr, topic: String) -> Self {
        Self::Kafka { addr, topic }
    }
}
