use runtime::prelude::*;

use crate::data::Auction;
use crate::data::Bid;
use crate::data::CompareOpV;
use crate::data::Value;
use crate::data::{Q6PrunedAuction, Q6PrunedBid};
use crate::WasmFunction;

#[data]
struct Output {
    seller: u64,
    avg_bid_price: u64,
}

#[data]
struct JoinOutput {
    auction_seller: u64,
    auction_expires: u64,
    auction_date_time: u64,
    bid_price: u64,
    bid_date_time: u64,
}

const TIME_SIZE: Duration = Duration::from_seconds(10);

const COUNT_SIZE: usize = 10;
const COUNT_SLIDE: usize = 1;

pub fn run(auctions: Stream<Auction>, bids: Stream<Bid>, ctx: &mut Context) {
    auctions
        .tumbling_window_join(
            ctx,
            bids,
            |a| a.id,
            |b| b.auction,
            TIME_SIZE,
            |a, b| JoinOutput::new(a.seller, a.expires, a.date_time, b.price, b.date_time),
        )
        .filter(ctx, |i| {
            i.auction_date_time < i.bid_date_time && i.bid_date_time < i.auction_expires
        })
        .keyby(ctx, |v| v.auction_seller)
        .count_sliding_holistic_window(ctx, COUNT_SIZE, COUNT_SLIDE, |seller, data| {
            let sum = data.iter().map(|v| v.bid_price).sum::<u64>();
            let count = data.len() as u64;
            Output::new(*seller, sum / count)
        })
        .drain(ctx);
}

// Opts:
// * Data pruning
pub fn run_opt(auctions: Stream<Auction>, bids: Stream<Bid>, ctx: &mut Context) {
    let auctions = auctions.map(ctx, |a| {
        Q6PrunedAuction::new(a.id, a.seller, a.expires, a.date_time)
    });
    let bids = bids.map(ctx, |b| Q6PrunedBid::new(b.auction, b.price, b.date_time));
    auctions
        .tumbling_window_join(
            ctx,
            bids,
            |a| a.id,
            |b| b.auction,
            TIME_SIZE,
            |a, b| JoinOutput::new(a.seller, a.expires, a.date_time, b.price, b.date_time),
        )
        .filter(ctx, |i| {
            i.auction_date_time < i.bid_date_time && i.bid_date_time < i.auction_expires
        })
        .keyby(ctx, |v| v.auction_seller)
        .count_sliding_holistic_window(ctx, COUNT_SIZE, COUNT_SLIDE, |seller, data| {
            let sum = data.iter().map(|v| v.bid_price).sum::<u64>();
            let count = data.len() as u64;
            Output::new(*seller, sum / count)
        })
        .drain(ctx);
}

// Wasm
pub fn run_wasm(auctions: Stream<Auction>, bids: Stream<Bid>, ctx: &mut Context, wasm_func1: WasmFunction<(Vec<CompareOpV>,), (bool,)>) {
    let auctions = auctions.map(ctx, |a| {
        Q6PrunedAuction::new(a.id, a.seller, a.expires, a.date_time)
    });
    let bids = bids.map(ctx, |b| Q6PrunedBid::new(b.auction, b.price, b.date_time));
    auctions
        .tumbling_window_join(
            ctx,
            bids,
            |a| a.id,
            |b| b.auction,
            TIME_SIZE,
            |a, b| JoinOutput::new(a.seller, a.expires, a.date_time, b.price, b.date_time),
        )
        .filter(ctx, move |i| {
            wasm_func1.call((vec![
                CompareOpV::Lt((Value::TyU64(i.auction_date_time), Value::TyU64(i.bid_date_time)),), 
                CompareOpV::Lt((Value::TyU64(i.bid_date_time), Value::TyU64(i.auction_expires)),)
            ],)).0
        })
        .keyby(ctx, |v| v.auction_seller)
        .count_sliding_holistic_window(ctx, COUNT_SIZE, COUNT_SLIDE, |seller, data| {
            let sum = data.iter().map(|v| v.bid_price).sum::<u64>();
            let count = data.len() as u64;
            Output::new(*seller, sum / count)
        })
        .drain(ctx);
}
