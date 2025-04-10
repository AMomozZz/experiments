wit_bindgen::generate!({
    world: "component",
});

use exports::pkg::component::nexmark::{Bid, Guest as NexmarkGuest, EitherData};
// use pkg::component::data_type::{Auction, Person};

struct Component;

export!(Component);

impl NexmarkGuest for Component {
    fn qs(bid: Bid,) -> Option<Bid> {
        Some(Bid {auction: bid.auction, price: bid.price * 100 / 85, bidder: bid.bidder, date_time: bid.date_time, channel: bid.channel, url: bid.url, extra: bid.extra })
    }
    
    fn qs_g(data: EitherData,) -> Option<EitherData> {
        Some(match data {
            // EitherData::Auction(auction) => {
            //     // let return_auction = Auction {id: auction.id, item_name: auction.item_name, description: auction.description, initial_bid: auction.initial_bid, reserve: auction.reserve, date_time: auction.date_time, expires: auction.expires, seller: auction.seller, category: auction.category, extra: auction.extra};

            //     EitherData::Auction(auction)
            // },
            EitherData::Bid(bid) => {
                let return_bid = Bid { auction: bid.auction, price: bid.price * 100 / 85, bidder: bid.bidder, date_time: bid.date_time, channel: bid.channel, url: bid.url, extra: bid.extra };

                EitherData::Bid(return_bid)
            },
            // EitherData::Person(person) => {
            //     // let return_person = Person { id: person.id, name: person.name, email_address: person.email_address, credit_card: person.credit_card, city: person.city, state: person.state, date_time: person.date_time, extra: person.extra };

            //     EitherData::Person(person)
            // },
            d => d,
        })
    }
}