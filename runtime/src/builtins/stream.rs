use crate::builtins::time::Time;
use crate::traits::Data;
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

pub mod assert;
pub mod batch;
pub mod drain;
pub mod filter;
pub mod filter_map;
pub mod flat_map;
pub mod fork;
pub mod join;
pub mod keyby;
pub mod map;
pub mod merge;
pub mod operator;
pub mod scan;
pub mod sink;
pub mod source;
pub mod take;
pub mod window;
pub mod collect;
pub mod sort;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Event<T> {
    Data(Time, T),
    Watermark(Time),
    Snapshot(usize),
    Sentinel,
}

#[must_use]
pub struct Stream<T>(pub(crate) Receiver<Event<T>>);

#[must_use]
pub struct Collector<T>(pub(crate) Sender<Event<T>>);

impl<T: Data> Stream<T> {
    pub async fn recv(&mut self) -> Event<T> {
        self.0.recv().await.unwrap_or(Event::Sentinel)
    }
}

impl<T: Data> Collector<T> {
    pub async fn send(&self, event: Event<T>) -> Result<(), SendError> {
        self.0.send(event).await.map_err(|_| SendError::Closed)
    }
}

impl<T> Stream<T> {
    pub fn new() -> (Collector<T>, Stream<T>) {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        (Collector(tx), Stream(rx))
    }
}

pub enum SendError {
    Closed,
}
