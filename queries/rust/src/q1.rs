use core::time;
use std::{process::exit, sync::{Arc, Mutex}};

use runtime::prelude::{stream::operator, *};
use wasmtime::{component::{Instance, Linker, TypedFunc}, Store};
use crate::{data::Bid, Host};
use wasmtime_wasi::{WasiCtx, WasiImpl};

#[data]
struct Output {
    auction: u64,
    price: u64,
    bidder: u64,
    date_time: u64,
}

pub fn run(bids: Stream<Bid>, ctx: &mut Context) {
    bids.map(ctx, |bid| {
        Output::new(bid.auction, bid.price * 100 / 85, bid.bidder, bid.date_time)
    })
    .drain(ctx);
}

pub fn run_opt(bids: Stream<Bid>, ctx: &mut Context) {
    bids.map(ctx, |bid| {
        Output::new(bid.auction, bid.price * 100 / 85, bid.bidder, bid.date_time)
    })
    .drain(ctx);
}
// 26784, 28243, 27930, 35737, 37400, 43479
// pub fn run_wasm(bids: Stream<Bid>, ctx: &mut Context, func_typed: TypedFunc<(u64, u64, u64, u64), ((u64, u64, u64, u64),)>, s: Arc<Mutex<Store<WasiImpl<Host>>>>) {
//     bids.map(ctx, move |bid| {
//         let mut store_guard  = s.lock().unwrap();
//         let store: &mut Store<WasiImpl<Host>> = &mut *store_guard;
        
//         // let intf_export = instance
//         //     .get_export(&mut *store, None, "pkg:component/nexmark")
//         //     .unwrap();
//         // let func_print_export = instance
//         //     .get_export(&mut *store, Some(&intf_export), "q1")
//         //     .unwrap();
//         // let func_print_typed = instance
//         //     .get_typed_func::<(u64, u64, u64, u64), ((u64, u64, u64, u64),)>(&mut *store, func_print_export)
//         //     .unwrap();

//         let ((auction, price, bidder, date_time),) =
//             func_typed.call(&mut *store, (bid.auction, bid.price, bid.bidder, bid.date_time))
//             .unwrap();
//         func_typed.post_return(store).unwrap();
//         Output::new(auction, price, bidder, date_time)
//     })
//     .drain(ctx);
// }
pub fn run_wasm(bids: Stream<Bid>, ctx: &mut Context, instance: Instance, s: Arc<Mutex<Store<WasiImpl<Host>>>>) {
    bids.map(ctx, move |bid| {
        let time = std::time::Instant::now();
        let mut store_guard  = s.lock().unwrap();
        let store: &mut Store<WasiImpl<Host>> = &mut *store_guard;
        // eprintln!("{}", time.elapsed().as_nanos());
        
        let intf_export = instance
            .get_export(&mut *store, None, "pkg:component/nexmark")
            .unwrap();
        let func_print_export = instance
            .get_export(&mut *store, Some(&intf_export), "q1")
            .unwrap();
        let func_print_typed = instance
            .get_typed_func::<(u64, u64, u64, u64), ((u64, u64, u64, u64),)>(&mut *store, func_print_export)
            .unwrap();
        // eprintln!("{}", time.elapsed().as_nanos());
        
        let ((auction, price, bidder, date_time),) =
            func_print_typed.call(&mut *store, (bid.auction, bid.price, bid.bidder, bid.date_time))
            .unwrap();
        func_print_typed.post_return(store).unwrap();
        // eprintln!("{}", time.elapsed().as_nanos());
        // exit(0);
        Output::new(auction, price, bidder, date_time)
    })
    .drain(ctx);
}