wit_bindgen::generate!({
    world: "component",
});

use exports::pkg::component::nexmark::{Bid, Guest as NexmarkGuest, PrunedBid};

struct Component;

export!(Component);

impl NexmarkGuest for Component {
    fn q2(auction:u64, price:u64, filters:Vec<u64>,) -> Option<(u64,u64,)> {
        match filters.contains(&auction) {
            true => Some((auction, price)),
            false => None,
        }
    }

    fn single_filter(p:u64, filters:Vec<u64>,) -> bool {
        filters.contains(&p)
    }

    fn e1(p:u64,) -> bool {
        if p == 1007
            || p == 1020
            || p == 2001
            || p == 2019
            || p == 1087
        {
            return true;
        }
        return false;
    }

    fn all_in_wasm(bid: Bid,) -> Option<PrunedBid> {
        let p = bid.auction;
        if p == 1007
            || p == 1020
            || p == 2001
            || p == 2019
            || p == 1087
        {
            return Some(PrunedBid {auction: bid.auction, price: bid.price})
        }
        return None
    }

    fn all_in_wasm_not_pruned(bid: Bid,) -> Option<Bid> {
        let p = bid.auction;
        if p == 1007
            || p == 1020
            || p == 2001
            || p == 2019
            || p == 1087
        {
            return Some(Bid {auction: bid.auction, price: bid.price, bidder: bid.bidder, date_time: bid.date_time, channel: bid.channel, url: bid.url, extra: bid.extra })
        }
        return None
    }
}