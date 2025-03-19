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

    #[doc = " filter"]
    fn q2(auction:u64, price:u64, filters:Vec<u64>,) -> Option<(u64,u64,)> {
        match filters.contains(&auction) {
            true => Some((auction, price)),
            false => None,
        }
    }

    #[doc = " single-filter"]
    fn single_filter(p:u64, filters:Vec<u64>,) -> bool {
        filters.contains(&p)
    }

    #[doc = " multi-filter"]
    fn multi_filter(v:Vec<(u64, Vec<u64>)>,) -> bool {
        for (p, filters) in v {
            match filters.contains(&p) {
                true => continue,
                false => return false,
            }
        }
        true
    }

    #[doc = " multi-filter-opt"]
    fn multi_filter_opt(v:Vec<(u64, Vec<u64>)>,) -> bool {
        v.into_iter().all(|(p, filters)| filters.contains(&p))
    }

    #[doc = " string-single-filter"]
    fn string_single_filter(p: String, filters: Vec<String>) -> bool {
        filters.contains(&p)
    }
}




