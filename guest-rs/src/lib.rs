wit_bindgen::generate!({
    world: "component",
});

use exports::pkg::component::nexmark::Guest as NexmarkGuest;

struct Component;

export!(Component);

impl NexmarkGuest for Component {
    #[doc = " convert-currency"]
    fn q1(auction:u64, price:u64, bidder:u64, date_time:u64,) -> (u64,u64,u64,u64,) {
        (auction, price * 100 / 85, bidder, date_time)
    }
}




