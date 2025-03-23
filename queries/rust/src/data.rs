use runtime::prelude::*;
use wasmtime::component::{ComponentType, Lower, Lift};

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