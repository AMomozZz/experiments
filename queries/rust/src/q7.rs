use runtime::prelude::*;

use crate::{data::{Bid, Q7PrunedBid}, WasmFunction};

#[data]
struct Output {
    auction: u64,
    price: u64,
    bidder: u64,
}

const SIZE: Duration = Duration::from_seconds(10);

pub fn run(bids: Stream<Bid>, ctx: &mut Context) {
    bids.keyby(ctx, |b| b.auction)
        .time_tumbling_holistic_window(ctx, SIZE, |auction, bids, _| {
            let bid = bids.iter().max_by_key(|b| b.price).unwrap();
            Output::new(*auction, bid.price, bid.bidder)
        })
        .drain(ctx);
}

// Opts:
// * Data pruning
pub fn run_opt(bids: Stream<Bid>, ctx: &mut Context) {
    let bids = bids.map(ctx, |b| Q7PrunedBid::new(b.auction, b.price, b.bidder));
    bids.keyby(ctx, |b| b.auction)
        .time_tumbling_holistic_window(ctx, SIZE, |auction, bids, _| {
            let bid = bids.iter().max_by_key(|b| b.price).unwrap();
            Output::new(*auction, bid.price, bid.bidder)
        })
        .drain(ctx);
}


// Wasm
pub fn run_wasm(bids: Stream<Bid>, ctx: &mut Context, wasm_func: WasmFunction<(Vec<Q7PrunedBid>,), (Q7PrunedBid,)>) {
    let bids = bids.map(ctx, |b| Q7PrunedBid::new(b.auction, b.price, b.bidder));
    bids.keyby(ctx, |b| b.auction)
        .time_tumbling_holistic_window(ctx, SIZE, move |auction, bids, _| {
            let (bid,) = wasm_func.call((bids.into_iter().cloned().collect::<Vec<Q7PrunedBid>>(),));
            Output::new(*auction, bid.price, bid.bidder)
        })
        .drain(ctx);
}