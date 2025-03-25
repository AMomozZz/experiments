use runtime::prelude::*;

use crate::{data::{Bid, Q5PrunedBid}, WasmFunction};

#[data]
struct Output {
    auction: u64,
}


const SIZE: Duration = Duration::from_minutes(5);
const SLIDE: Duration = Duration::from_minutes(1);

pub fn run(bids: Stream<Bid>, ctx: &mut Context) {
    bids.keyby(ctx, |b| b.auction)
        .time_sliding_aligned_holistic_window(ctx, SIZE, SLIDE, |auction, bids, _| {
            (*auction, bids.into_iter().count())
        })
        .unkey(ctx)
        .time_sliding_aligned_holistic_window(ctx, SIZE, SLIDE, |items, _| {
            let auction = items.iter().max_by_key(|(_, a)| a).unwrap().0;
            Output::new(auction)
        })
        .drain(ctx);
}

// Opts:
// * Data pruning
pub fn run_opt(bids: Stream<Bid>, ctx: &mut Context) {
    let bids = bids.map(ctx, |b| Q5PrunedBid::new(b.auction, b.bidder));
    bids.keyby(ctx, |b| b.auction)
        .time_sliding_aligned_holistic_window(ctx, SIZE, SLIDE, |auction, bids, _| {
            (*auction, bids.into_iter().count())
        })
        .unkey(ctx)
        .time_sliding_aligned_holistic_window(ctx, SIZE, SLIDE, |items, _| {
            let auction = items.iter().max_by_key(|(_, a)| a).unwrap().0;
            Output::new(auction)
        })
        .drain(ctx);
}

// Wasm
pub fn run_wasm(bids: Stream<Bid>, ctx: &mut Context, wasm_func1: WasmFunction<(Vec<Q5PrunedBid>,), (u64,)>, wasm_func2: WasmFunction<(Vec<(u64, u64)>,), (u64,)>) {
    let bids = bids.map(ctx, |b| Q5PrunedBid::new(b.auction, b.bidder));
    bids.keyby(ctx, |b| b.auction)
        .time_sliding_aligned_holistic_window(ctx, SIZE, SLIDE, move |auction, bids, _| {
            (*auction, wasm_func1.call((bids.into_iter().cloned().collect::<Vec<Q5PrunedBid>>(), )).0)
        })
        .unkey(ctx)
        .time_sliding_aligned_holistic_window(ctx, SIZE, SLIDE, move |items, _| {
            let (auction,) = wasm_func2.call((items.into_iter().cloned().collect::<Vec<(u64, u64)>>(), ));
            Output::new(auction)
        })
        .drain(ctx);
}