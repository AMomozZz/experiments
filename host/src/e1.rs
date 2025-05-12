use runtime::prelude::*;
use crate::{data::{Bid, PrunedBid}, wasm::WasmFunction};

#[data]
pub struct Output {
    auction: u64,
    price: u64,
}

pub fn opt_func(bid: Bid) -> std::option::Option<Output> {
    if bid.auction == 1007
        || bid.auction == 1020
        || bid.auction == 2001
        || bid.auction == 2019
        || bid.auction == 1087
    {
        Option::Some(Output::new(bid.auction, bid.price))
    } else {
        Option::None
    }
}

// Opt:
// * Fusion
pub fn run_opt(bids: Stream<Bid>, ctx: &mut Context) {
    bids.filter_map(ctx, |bid| {
        opt_func(bid)
    })
    .drain(ctx);
}

// Wasm
pub fn run_wasm(bids: Stream<Bid>, ctx: &mut Context, wasm_func: WasmFunction<(u64, u64, Vec<u64>), (Option<(u64, u64)>,)>) {
    let v: Vec<u64> = vec![1007, 1020, 2001, 2019, 1087];

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
    let v: Vec<u64> = vec![1007, 1020, 2001, 2019, 1087];

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

pub fn run_wasm_e1(bids: Stream<Bid>, ctx: &mut Context, wasm_func: WasmFunction<(u64,), (bool,)>) {
    bids.filter_map(ctx, move |bid| {
        let (result,) = wasm_func.call((bid.auction,));
        // result
        match result {
            true => Option::Some(Output::new(bid.auction, bid.price)),
            false => Option::None,
        }
    })
    .drain(ctx);
}

pub fn run_wasm_e1_all_in_wasm_g<T>(bids: Stream<T>, ctx: &mut Context, wasm_func: WasmFunction<(T,), (Option<T>,)>)
where 
T: Clone + Unpin + for<'a> runtime::prelude::serde::Deserialize<'a> + runtime::prelude::serde::Serialize + std::fmt::Debug + std::marker::Send+ std::marker::Sync + wasmtime::component::Lower + wasmtime::component::ComponentType + wasmtime::component::Lift + 'static
{
    bids.filter_map(ctx, move |bid| {
        wasm_func.call((bid.clone(),)).0
    })
    .drain(ctx);
}

pub fn run_wasm_e1_all_in_wasm(bids: Stream<Bid>, ctx: &mut Context, wasm_func: WasmFunction<(Bid,), (Option<PrunedBid>,)>) {
    bids.filter_map(ctx, move |bid| {
        wasm_func.call((bid.clone(),)).0
    })
    .drain(ctx);
}