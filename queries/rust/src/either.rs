use runtime::prelude::serde::{Serialize, Deserialize};

use crate::data::{Auction, Bid, Person, WasmComponent};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "runtime::prelude::serde")]
pub enum Either {
    Component(WasmComponent),
    Bid(Bid),
    Auction(Auction),
    Person(Person),
}
