wit_bindgen::generate!({
    world: "component",
});

use exports::pkg::component::nexmark::{Auction, Bid, CompareOpV, Guest as NexmarkGuest, Q4Auction, Q4Bid, Q5Bid, Q6JoinOutput, Q7Bid, PrunedBid};
use exports::pkg::component::u64_compare::Guest as U64CompareGuest;
use crate::pkg::component::data_type::Value;

struct Component;

export!(Component);

impl NexmarkGuest for Component {
    #[doc = "convert-currency"]
    fn q1(auction:u64, price:u64, bidder:u64, date_time:u64,) -> (u64,u64,u64,u64,) {
        (auction, price * 100 / 85, bidder, date_time)
    }
    // fn q1(bid: Bid,) -> Bid {
    //     Bid {auction: bid.auction, price: bid.price * 100 / 85, bidder: bid.bidder, date_time: bid.date_time, channel: bid.channel, url: bid.url, extra: bid.extra }
    // }

    #[doc = "filter"]
    fn q2(auction:u64, price:u64, filters:Vec<u64>,) -> Option<(u64,u64,)> {
        match filters.contains(&auction) {
            true => Some((auction, price)),
            false => None,
        }
    }

    #[doc = "single-filter"]
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

    #[doc = "multi-filter"]
    fn multi_filter(v:Vec<(u64, Vec<u64>)>,) -> bool {
        for (p, filters) in v {
            match filters.contains(&p) {
                true => continue,
                false => return false,
            }
        }
        true
    }

    #[doc = "multi-filter-opt"]
    fn multi_filter_opt(v:Vec<(u64, Vec<u64>)>,) -> bool {
        v.into_iter().all(|(p, filters)| filters.contains(&p))
    }

    #[doc = "string-single-filter"]
    fn string_single_filter(p: String, filters: Vec<String>) -> bool {
        filters.contains(&p)
    }
    
    #[doc = " single-less-or-equal"]
    fn less_or_equal_single(a:u64,b:u64,) -> bool {
        a <= b
    }
    
    #[doc = " multi-less-or-equal"]
    fn less_or_equal_multi(v: Vec<(u64,u64,)>,) -> bool {
        v.into_iter().all(|(a,b)| a <= b)
    }
    
    #[doc = " q4-max-of-bid-price"]
    fn q4_max_of_bid_price(v: Vec<(Auction, Bid,)>,) -> u64 {
        v.iter().map(|(_, b)| b.price).max().unwrap()
    }

    #[doc = "q4-max-of-bid-price-pruned"]
    fn q4_max_of_bid_price_p(v: Vec<(Q4Auction, Q4Bid,)>,) -> u64 {
        v.iter().map(|(_, b)| b.price).max().unwrap()
    }

    #[doc = "q4-avg"]
    fn q4_avg(v: Vec<(u64, u64,)>,) -> u64 {
        let sum = v.iter().map(|(_, max)| max).sum::<u64>();
        let count = v.len() as u64;
        sum / count
    }
    
    #[doc = " q5-count"]
    fn q5_count(v:Vec<Q5Bid>,) -> u64 {
        v.iter().count() as u64
    }

    #[doc = "q5-max-by-key"]
    fn q5_max_by_key(v: Vec<(u64, u64)>,) -> u64 {
        v.iter().max_by_key(|(_, a)| a).unwrap().0
    }
    
    #[doc = " q6-multi-comparison"]
    fn q6_multi_comparison_v(v: Vec<CompareOpV>,) -> bool {
        v.into_iter().all(|op| {
            match op {
                CompareOpV::Eq((Value::TyU64(a), Value::TyU64(b)),) => Some(a == b),
                CompareOpV::Eq((Value::TyString(a), Value::TyString(b)),) => Some(a == b),
                
                CompareOpV::Ne((Value::TyU64(a), Value::TyU64(b)),) => Some(a != b),
                CompareOpV::Ne((Value::TyString(a), Value::TyString(b)),) => Some(a != b),
        
                CompareOpV::Gt((Value::TyU64(a), Value::TyU64(b)),) => Some(a > b),
                CompareOpV::Gte((Value::TyU64(a), Value::TyU64(b)),) => Some(a >= b),
                CompareOpV::Lt((Value::TyU64(a), Value::TyU64(b)),) => Some(a < b),
                CompareOpV::Lte((Value::TyU64(a), Value::TyU64(b)),) => Some(a <= b),
                _ => None,
            }.unwrap_or(false)
        })
    }

    #[doc = "q6-avg"]
    fn q6_avg(v: Vec<Q6JoinOutput>) -> u64 {
        let sum = v.iter().map(|v| v.bid_price).sum::<u64>();
        let count = v.len() as u64;
        sum / count
    }

    #[doc = "q7"]
    fn q7(v: Vec<Q7Bid>) -> Q7Bid {
        *v.iter().max_by_key(|b| b.price).unwrap()
    }


}

impl U64CompareGuest for Component {
    fn eq(value1:u64,value2:u64,) -> bool {
        value1 == value2
    }

    fn ne(value1:u64,value2:u64,) -> bool {
        value1 != value2
    }

    fn gt(value1:u64,value2:u64,) -> bool {
        value1 > value2
    }

    fn gte(value1:u64,value2:u64,) -> bool {
        value1 >= value2
    }

    fn lt(value1:u64,value2:u64,) -> bool {
        value1 < value2
    }

    fn lte(value1:u64,value2:u64,) -> bool {
        value1 <= value2
    }
    
    fn eq_m(v: Vec<(u64,u64,)>,) -> bool {
        v.into_iter().all(|(a,b)| a == b)
    }
    
    fn ne_m(v: Vec<(u64,u64,)>,) -> bool {
        v.into_iter().all(|(a,b)| a != b)
    }
    
    fn gt_m(v: Vec<(u64,u64,)>,) -> bool {
        v.into_iter().all(|(a,b)| a > b)
    }
    
    fn gte_m(v: Vec<(u64,u64,)>,) -> bool {
        v.into_iter().all(|(a,b)| a >= b)
    }
    
    fn lt_m(v: Vec<(u64,u64,)>,) -> bool {
        v.into_iter().all(|(a,b)| a < b)
    }
    
    fn lte_m(v: Vec<(u64,u64,)>,) -> bool {
        v.into_iter().all(|(a,b)| a <= b)
    }
}