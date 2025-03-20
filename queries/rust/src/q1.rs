use std::cell::RefCell;
use runtime::prelude::*;
use wasmtime::{component::TypedFunc, Store};
use crate::{data::Bid, Host, WasmFunction};
use wasmtime_wasi::WasiImpl;

#[data]
struct Output {
    auction: u64,
    price: u64,
    bidder: u64,
    date_time: u64,
}

pub fn run(bids: Stream<Bid>, ctx: &mut Context) {
    bids.map(ctx, |bid| {
        Output::new(bid.auction, bid.price * 100 / 85, bid.bidder, bid.date_time)
    })
    .drain(ctx);
}

pub fn run_opt(bids: Stream<Bid>, ctx: &mut Context) {
    bids.map(ctx, |bid| {
        Output::new(bid.auction, bid.price * 100 / 85, bid.bidder, bid.date_time)
    })
    .drain(ctx);
}

pub fn run_wasm(bids: Stream<Bid>, ctx: &mut Context, wasm_func: WasmFunction<(u64, u64, u64, u64), ((u64, u64, u64, u64),)>) {
    bids.map(ctx, move |bid| {
        let ((auction, price, bidder, date_time),) = wasm_func.call((bid.auction, bid.price, bid.bidder, bid.date_time));
        Output::new(auction, price, bidder, date_time)
    })
    .drain(ctx);
}