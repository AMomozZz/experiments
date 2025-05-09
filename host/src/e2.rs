use runtime::prelude::*;
use crate::{data::Bid, wasm::WasmFunction};

#[data]
struct Output {
    auction: u64,
    price: u64,
}

// Opt:
// * Fusion
pub fn run_opt(bids: Stream<Bid>, ctx: &mut Context) {
    bids.filter_map(ctx, |bid| {
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
    })
    .drain(ctx);
}

pub fn run_wasm_e2(bids: Stream<Bid>, ctx: &mut Context, wasm_func: WasmFunction<(u64,), (bool,)>) {
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