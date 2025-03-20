use runtime::prelude::*;
use wasmtime::{component::TypedFunc, Store};
use crate::{data::Bid, Host, WasmFunction};
use wasmtime_wasi::WasiImpl;
use std::cell::RefCell;

#[data]
struct Output {
    auction: u64,
    price: u64,
}

pub fn run(bids: Stream<Bid>, ctx: &mut Context) {
    bids.filter(ctx, |bid| {
        bid.auction == 1007
            || bid.auction == 1020
            || bid.auction == 2001
            || bid.auction == 2019
            || bid.auction == 2087
    })
    .map(ctx, |bid| Output::new(bid.auction, bid.price))
    .drain(ctx);
}

// Opt:
// * Fusion
pub fn run_opt(bids: Stream<Bid>, ctx: &mut Context) {
    bids.filter_map(ctx, |bid| {
        if bid.auction == 1007
            || bid.auction == 1020
            || bid.auction == 2001
            || bid.auction == 2019
            || bid.auction == 2087
        {
            Option::Some(Output::new(bid.auction, bid.price))
        } else {
            Option::None
        }
    })
    .drain(ctx);
}

// Wasm
pub fn run_wasm(bids: Stream<Bid>, ctx: &mut Context, wasm_func: WasmFunction<(u64, u64, Vec<u64>), (Option<(u64, u64)>,)>) {
    let v: Vec<u64> = vec![1007, 1020, 2001, 2019, 2087];

    bids.filter_map(ctx, move |bid| {
        let (result,) = wasm_func.call((bid.auction, bid.price, v.clone()));
        match result {
            Some((auction, price)) => Option::Some(Output::new(auction, price)),
            None => Option::None,
        }
    })
    .drain(ctx);
}

pub fn run_wasm_sf(bids: Stream<Bid>, ctx: &mut Context, wasm_func: WasmFunction<(u64, Vec<u64>,), (bool,)>) {
    let v: Vec<u64> = vec![1007, 1020, 2001, 2019, 2087];

    bids.filter_map(ctx, move |bid| {
        let (result,) = wasm_func.call((bid.auction, v.clone()));
        // result
        match result {
            true => Option::Some(Output::new(bid.auction, bid.price)),
            false => Option::None,
        }
    })
    .drain(ctx);
}

pub fn run_wasm_mf(bids: Stream<Bid>, ctx: &mut Context, wasm_func: WasmFunction<(Vec<(u64, Vec<u64>)>,), (bool,)>) {
    let v: Vec<u64> = vec![1007, 1020, 2001, 2019, 2087];

    bids.filter_map(ctx, move |bid| {
        let (result,) = wasm_func.call((vec![(bid.auction, v.clone())],));
        // result
        match result {
            true => Option::Some(Output::new(bid.auction, bid.price)),
            false => Option::None,
        }
    })
    .drain(ctx);
}
