use runtime::prelude::*;
use wasmtime::component::{ComponentType, Lift, Lower};

use crate::{data::{Auction, Bid, Person}, wasm::WasmComponent};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(crate = "runtime::prelude::serde")]
pub enum Either {
    Component(WasmComponent),
    // Bid(Bid),
    // Auction(Auction),
    // Person(Person),
    Data(EitherData),
}


#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, ComponentType, Lower, Lift)]
#[serde(crate = "runtime::prelude::serde")]
#[component(variant)]
pub enum EitherData {
    Bid(Bid),
    Auction(Auction),
    Person(Person),
}