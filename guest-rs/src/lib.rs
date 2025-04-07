wit_bindgen::generate!({
    world: "component",
});

use exports::pkg::component::nexmark::{Bid, Guest as NexmarkGuest};

struct Component;

export!(Component);

impl NexmarkGuest for Component {
    fn qs(bid: Bid,) -> Option<Bid> {
        let filters = vec![1007, 1020, 2001, 2019, 2087];
        match filters.contains(&bid.auction) {
            true => Some(bid),
            false => None,
        }
    }
}
