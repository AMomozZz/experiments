use std::vec;

use runtime::prelude::*;

use crate::data::{Auction, Bid, Q4PrunedAuction, Q4PrunedBid};
use crate::WasmFunction;

#[data]
struct Output {
    category: u64,
    avg_price: u64,
}

const SIZE: Duration = Duration::from_seconds(10);

pub fn run(auctions: Stream<Auction>, bids: Stream<Bid>, ctx: &mut Context) {
    auctions
        .tumbling_window_join(
            ctx,
            bids,
            |auction| auction.id,
            |bid| bid.auction,
            SIZE,
            |auction, bid| (auction.clone(), bid.clone()),
        )
        .filter(ctx, |(a, b)| {
            a.date_time <= b.date_time && b.date_time <= a.expires
        })
        .keyby(ctx, |(a, _)| (a.id, a.category))
        .time_tumbling_holistic_window(ctx, SIZE, |(_, category), items, _| {
            let max = items.iter().map(|(_, b)| b.price).max().unwrap();
            (*category, max)
        })
        .keyby(ctx, |(_, category)| *category)
        .time_tumbling_holistic_window(ctx, SIZE, |category, items, _| {
            let sum = items.iter().map(|(_, max)| max).sum::<u64>();
            let count = items.len() as u64;
            Output::new(*category, sum / count)
        })
        .drain(ctx);
}

// Opts:
// * Data pruning
pub fn run_opt(auctions: Stream<Auction>, bids: Stream<Bid>, ctx: &mut Context) {
    let auctions = auctions.map(ctx, |a| {
        Q4PrunedAuction::new(a.id, a.category, a.expires, a.date_time)
    });
    let bids = bids.map(ctx, |b| Q4PrunedBid::new(b.auction, b.price, b.date_time));

    auctions
        .tumbling_window_join(
            ctx,
            bids,
            |auction| auction.id,
            |bid| bid.auction,
            SIZE,
            |auction, bid| (auction.clone(), bid.clone()),
        )
        .filter(ctx, |(a, b)| {
            a.date_time <= b.date_time && b.date_time <= a.expires
        })
        .keyby(ctx, |(a, _)| (a.id, a.category))
        .time_tumbling_holistic_window(ctx, SIZE, |(_, category), items, _| {
            let max = items.iter().map(|(_, b)| b.price).max().unwrap();
            (*category, max)
        })
        .keyby(ctx, |(_, category)| *category)
        .time_tumbling_holistic_window(ctx, SIZE, |category, items, _| {
            let sum = items.iter().map(|(_, max)| max).sum::<u64>();
            let count = items.len() as u64;
            Output::new(*category, sum / count)
        })
        .drain(ctx);
}


// Wasm
pub fn run_wasm(
    auctions: Stream<Auction>, 
    bids: Stream<Bid>, 
    ctx: &mut Context, 
    // wasm_func1: WasmFunction<(u64, u64), (bool,)>, 
    wasm_func1: WasmFunction<(Vec<(u64, u64)>,), (bool,)>, 
    wasm_func2: WasmFunction<(Vec<(Auction, Bid)>,), (u64,)>, 
    wasm_func3: WasmFunction<(Vec<(u64, u64)>,), (u64,)>) {
    // let auctions = auctions.map(ctx, |a| {
    //     Q4PrunedAuction::new(a.id, a.category, a.expires, a.date_time)
    // });
    // let bids = bids.map(ctx, |b| Q4PrunedBid::new(b.auction, b.price, b.date_time));

    auctions
        .tumbling_window_join(
            ctx,
            bids,
            |auction| auction.id,
            |bid| bid.auction,
            SIZE,
            |auction, bid| (auction.clone(), bid.clone()),
        )
        .filter(ctx, move |(a, b)| {
            // wasm_func1.call((a.date_time, b.date_time)).0 && wasm_func1.call((b.date_time, a.expires)).0
            wasm_func1.call((vec![(a.date_time, b.date_time), (b.date_time, a.expires)], )).0
        })
        .keyby(ctx, |(a, _)| (a.id, a.category))
        .time_tumbling_holistic_window(ctx, SIZE, move |(_, category), items, _| {
            let (max,) = wasm_func2.call((items.to_vec(),));
            (*category, max)
        })
        .keyby(ctx, |(_, category)| *category)
        .time_tumbling_holistic_window(ctx, SIZE, move |category, items, _| {
            let (avg,) = wasm_func3.call((items.to_vec(),));
            Output::new(*category, avg)
        })
        .drain(ctx);
}

pub fn run_wasm_p(
    auctions: Stream<Auction>, 
    bids: Stream<Bid>, 
    ctx: &mut Context, 
    wasm_func1: WasmFunction<(Vec<(u64, u64)>,), (bool,)>, 
    wasm_func2: WasmFunction<(Vec<(Q4PrunedAuction, Q4PrunedBid)>,), (u64,)>, 
    wasm_func3: WasmFunction<(Vec<(u64, u64)>,), (u64,)>) {
    let auctions = auctions.map(ctx, |a| {
        Q4PrunedAuction::new(a.id, a.category, a.expires, a.date_time)
    });
    let bids = bids.map(ctx, |b| Q4PrunedBid::new(b.auction, b.price, b.date_time));

    auctions
        .tumbling_window_join(
            ctx,
            bids,
            |auction| auction.id,
            |bid| bid.auction,
            SIZE,
            |auction, bid| (auction.clone(), bid.clone()),
        )
        .filter(ctx, move |(a, b)| {
            wasm_func1.call((vec![(a.date_time, b.date_time), (b.date_time, a.expires)], )).0
        })
        .keyby(ctx, |(a, _)| (a.id, a.category))
        .time_tumbling_holistic_window(ctx, SIZE, move |(_, category), items, _| {
            let (max,) = wasm_func2.call((items.to_vec(),));
            (*category, max)
        })
        .keyby(ctx, |(_, category)| *category)
        .time_tumbling_holistic_window(ctx, SIZE, move |category, items, _| {
            let (avg,) = wasm_func3.call((items.to_vec(),));
            Output::new(*category, avg)
        })
        .drain(ctx);
}
