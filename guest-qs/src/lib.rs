wit_bindgen::generate!({
    world: "component",
});

use exports::pkg::component::nexmark::{Bid, Guest as NexmarkGuest};

struct Component;

export!(Component);

impl NexmarkGuest for Component {
    fn qs(bid: Bid,) -> Option<Bid> {
        Some(Bid {auction: bid.auction, price: bid.price * 100 / 85, bidder: bid.bidder, date_time: bid.date_time, channel: bid.channel, url: bid.url, extra: bid.extra })
    }
}