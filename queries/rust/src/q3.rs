use std::cell::RefCell;

use runtime::prelude::*;
use wasmtime::component::TypedFunc;
use wasmtime::Store;
use wasmtime_wasi::WasiImpl;

use crate::data::Auction;
use crate::data::Person;
use crate::Host;

#[data]
struct Output {
    name: String,
    city: String,
    state: String,
    id: u64,
}

#[data]
struct PrunedAuction {
    id: u64,
    seller: u64,
}

#[data]
struct PrunedPerson {
    id: u64,
    name: String,
    city: String,
    state: String,
}

const SIZE: Duration = Duration::from_seconds(10);

pub fn run(auctions: Stream<Auction>, persons: Stream<Person>, ctx: &mut Context) {
    auctions
        .tumbling_window_join(
            ctx,
            persons,
            |auction| auction.seller,
            |person| person.id,
            SIZE,
            |auction, person| (auction.clone(), person.clone()),
        )
        .filter(ctx, |(auction, person)| {
            (person.state == "or" || person.state == "id" || person.state == "ca")
                && auction.category == 10
        })
        .map(ctx, |(auction, person)| {
            Output::new(person.name, person.city, person.state, auction.id)
        })
        .drain(ctx);
}

// Opts:
// * Data pruning
// * Predicate pushdown
// * Operator fusion
pub fn run_opt(auctions: Stream<Auction>, persons: Stream<Person>, ctx: &mut Context) {
    let persons2 = persons.filter_map(ctx, |p| {
        if p.state == "or" || p.state == "id" || p.state == "ca" {
            Option::Some(PrunedPerson::new(p.id, p.name, p.city, p.state))
        } else {
            Option::None
        }
    });
    let auctions2 = auctions.filter_map(ctx, |a| {
        if a.category == 10 {
            Option::Some(PrunedAuction::new(a.id, a.seller))
        } else {
            Option::None
        }
    });
    auctions2
        .tumbling_window_join(
            ctx,
            persons2,
            |a| a.seller,
            |p| p.id,
            SIZE,
            |a, p| Output::new(p.name.clone(), p.city.clone(), p.state.clone(), a.id),
        )
        .drain(ctx);
}

// Wasm
pub fn run_wasm(auctions: Stream<Auction>, persons: Stream<Person>, ctx: &mut Context, string_filter_func: TypedFunc<(String, Vec<String>,), (bool,)>, number_filter_func: TypedFunc<(u64, Vec<u64>,), (bool,)>, store_wrapper: RefCell<Store<WasiImpl<Host>>>) {
    let v1: Vec<String> = vec!["or".to_string(), "id".to_string(), "ca".to_string()];
    let v2: Vec<u64> = vec![10];

    let persons2 = {
        persons.filter_map(ctx, move |p| {
            let mut store = store_wrapper.borrow_mut();
            let (result,) =
                string_filter_func.call(&mut *store, (p.state.clone(), v1.clone(),))
                .unwrap();
            string_filter_func.post_return(&mut *store).unwrap();
            // result
            match result {
                true => Option::Some(PrunedPerson::new(p.id, p.name, p.city, p.state)),
                false => Option::None,
            }
        })
    };
    
    let auctions2 = {
        auctions.filter_map(ctx, move |a| {
            let mut store = store_wrapper.borrow_mut();
            let (result,) =
                number_filter_func.call(&mut *store, (a.category.clone(), v2.clone(),))
                .unwrap();
            number_filter_func.post_return(&mut *store).unwrap();
            // result
            match result {
                true => Option::Some(PrunedAuction::new(a.id, a.seller)),
                false => Option::None,
            }
        })
    };
    
    auctions2
        .tumbling_window_join(
            ctx,
            persons2,
            |a| a.seller,
            |p| p.id,
            SIZE,
            |a, p| Output::new(p.name.clone(), p.city.clone(), p.state.clone(), a.id),
        )
        .drain(ctx);
}