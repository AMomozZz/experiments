use std::cell::RefCell;
use runtime::prelude::*;
use wasmtime::{component::TypedFunc, Store};
use crate::{data::Bid, Host};
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

pub fn run_wasm(bids: Stream<Bid>, ctx: &mut Context, func_typed: TypedFunc<(u64, u64, u64, u64), ((u64, u64, u64, u64),)>, store_wrapper: RefCell<Store<WasiImpl<Host>>>) {
    bids.map(ctx, move |bid| {
        let mut store = store_wrapper.borrow_mut();
        let ((auction, price, bidder, date_time),) =
            func_typed.call(&mut *store, (bid.auction, bid.price, bid.bidder, bid.date_time))
            .unwrap();
        func_typed.post_return(&mut *store).unwrap();
        Output::new(auction, price, bidder, date_time)
    })
    .drain(ctx);
}