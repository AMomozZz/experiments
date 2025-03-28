use std::net::SocketAddr;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::traits::DeepClone;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[repr(C)]
pub enum Writer {
    Stdout,
    File { path: PathBuf },
    Tcp { addr: SocketAddr },
    Kafka { addr: SocketAddr, topic: String },
}

impl DeepClone for Writer {
    fn deep_clone(&self) -> Self {
        self.clone()
    }
}

impl std::fmt::Display for Writer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Writer::Stdout => write!(f, "stdout()"),
            Writer::File { path } => write!(f, "file(path={})", path.display()),
            // Writer::Http { url } => write!(f, "http(url={url})"),
            Writer::Tcp { addr } => write!(f, "tcp(addr={addr})"),
            Writer::Kafka { addr, topic } => write!(f, "kafka(addr={addr}, topic={topic})"),
        }
    }
}

impl Writer {
    pub fn stdout() -> Self {
        Self::Stdout
    }
    pub fn file(path: PathBuf) -> Self {
        Self::File { path }
    }
    pub fn tcp(addr: SocketAddr) -> Self {
        Self::Tcp { addr }
    }
    pub fn kafka(addr: SocketAddr, topic: String) -> Self {
        Self::Kafka { addr, topic }
    }
}
