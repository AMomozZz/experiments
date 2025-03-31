use runtime::prelude::*;
use wasmtime::component::{ComponentType, Lift, Lower};

// use crate::WasmFunction;

#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, New, ComponentType, Lower, Lift)]
#[serde(crate = "runtime::prelude::serde")]
#[component(record)]
pub struct Auction {
    pub id: u64,
    #[component(name = "item-name")]
    pub item_name: String,
    pub description: String,
    #[component(name = "initial-bid")]
    pub initial_bid: u64,
    pub reserve: u64,
    #[timestamp]
    #[component(name = "date-time")]
    pub date_time: u64,
    pub expires: u64,
    pub seller: u64,
    pub category: u64,
    pub extra: String,
}

#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, New, ComponentType, Lower, Lift)]
#[serde(crate = "runtime::prelude::serde")]
#[component(record)]
pub struct Person {
    pub id: u64,
    pub name: String,
    #[component(name = "email-address")]
    pub email_address: String,
    #[component(name = "credit-card")]
    pub credit_card: String,
    pub city: String,
    pub state: String,
    #[timestamp]
    #[component(name = "date-time")]
    pub date_time: u64,
    pub extra: String,
}

#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, New, ComponentType, Lower, Lift)]
#[serde(crate = "runtime::prelude::serde")]
#[component(record)]
pub struct Bid {
    pub auction: u64,
    pub bidder: u64,
    pub price: u64,
    pub channel: String,
    pub url: String,
    #[timestamp]
    #[component(name = "date-time")]
    pub date_time: u64,
    pub extra: String,
}

// #[data]
// pub struct MyComponent<I:std::fmt::Debug, O:std::fmt::Debug> {
//     component: WasmFunction<I, O>,
//     #[timestamp]
//     date_time: u64,
// }

#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, New, ComponentType, Lower, Lift)]
#[serde(crate = "runtime::prelude::serde")]
#[component(record)]
pub struct Q4PrunedAuction {
    pub id: u64,
    pub category: u64,
    pub expires: u64,
    #[component(name = "date-time")]
    pub date_time: u64,
}

#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, New, ComponentType, Lower, Lift)]
#[serde(crate = "runtime::prelude::serde")]
#[component(record)]
pub struct Q4PrunedBid {
    pub auction: u64,
    pub price: u64,
    #[component(name = "date-time")]
    pub date_time: u64,
}

#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, New, ComponentType, Lower, Lift)]
#[serde(crate = "runtime::prelude::serde")]
#[component(record)]
pub struct Q5PrunedBid {
    pub auction: u64,
    pub bidder: u64,
}

#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, New, ComponentType, Lower, Lift)]
#[serde(crate = "runtime::prelude::serde")]
#[component(record)]
pub struct Q6PrunedAuction {
    pub id: u64,
    pub seller: u64,
    pub expires: u64,
    #[component(name = "date-time")]
    pub date_time: u64,
}

#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, New, ComponentType, Lower, Lift)]
#[serde(crate = "runtime::prelude::serde")]
#[component(record)]
pub struct Q6PrunedBid {
    pub auction: u64,
    pub price: u64,
    #[component(name = "date-time")]
    pub date_time: u64,
}

#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, ComponentType, Lower, Lift)]
#[serde(crate = "runtime::prelude::serde")]
#[component(variant)]
pub enum Value {
    #[component(name = "ty-u64")]
    TyU64(u64),
    #[component(name = "ty-string")]
    TyString(String),
}

#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, ComponentType, Lower, Lift)]
#[serde(crate = "runtime::prelude::serde")]
#[component(variant)]
pub enum CompareOpV {
    #[component(name = "eq")]
    Eq((Value, Value),),   // ==ne
    #[component(name = "ne")]
    Ne((Value, Value),),   // !=
    #[component(name = "gt")]
    Gt((Value, Value),),   // >
    #[component(name = "gte")]
    Gte((Value, Value),),  // >=
    #[component(name = "lt")]
    Lt((Value, Value),),   // <
    #[component(name = "lte")]
    Lte((Value, Value),),  // <=
    // Contains((Value, Vec<Value>),),
}

#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, New, ComponentType, Lower, Lift)]
#[serde(crate = "runtime::prelude::serde")]
#[component(record)]
pub struct Q6JoinOutput {
    #[component(name = "auction-seller")]
    pub auction_seller: u64,
    #[component(name = "auction-expires")]
    pub auction_expires: u64,
    #[component(name = "auction-date-time")]
    pub auction_date_time: u64,
    #[component(name = "bid-price")]
    pub bid_price: u64,
    #[component(name = "bid-date-time")]
    pub bid_date_time: u64,
}

#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, New, ComponentType, Lower, Lift)]
#[serde(crate = "runtime::prelude::serde")]
#[component(record)]
pub struct Q7PrunedBid {
    pub auction: u64,
    pub price: u64,
    pub bidder: u64,
}

#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, New, ComponentType, Lower, Lift)]
#[serde(crate = "runtime::prelude::serde")]
#[component(record)]
pub struct QwOutput {
    pub mean: f64,
    pub stddev: f64,
    pub min: u64,
    pub max: u64,
}

#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, New, ComponentType, Lower, Lift)]
#[serde(crate = "runtime::prelude::serde")]
#[component(record)]
pub struct QwPrunedBid {
    pub price: u64,
}