use runtime::prelude::*;
use wasmtime::Store;
use crate::data::Bid;
use wasmtime_wasi::WasiImpl;

const GUEST_RS_WASI_MODULE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../guest-rs/target/wasm32-wasip2/release/component.wasm"
));

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
// pub fn run_wasm(bids: Stream<Bid>, store: &mut Store<WasiImpl>) {
//     bids.map(bids.ctx, move |bid| {
//         let instance = linker.instantiate(&mut store, &component).unwrap();
//         let intf_export = instance
//             .get_export(&mut store, None, "pkg:component/nexmark")
//             .unwrap();
//         let func_print_export = instance
//             .get_export(&mut store, Some(&intf_export), "q1")
//             .unwrap();
//         let func_print_typed = instance
//             .get_typed_func::<(u64, u64, u64, u64), ((u64, u64, u64, u64),)>(&mut store, func_print_export)
//             .unwrap();

//         let ((auction, price, bidder, date_time),) =
//             func_print_typed.call(&mut store, (bid.auction, bid.price, bid.bidder, bid.date_time))
//             .unwrap();
//         func_print_typed.post_return(store).unwrap();
//         Output::new(auction, price, bidder, date_time)
//     })
//     .drain(ctx);
// }
pub fn run_wasm(bids: Stream<Bid>, ctx: &mut Context) { 
    let engine = Engine::default();
    let component = Component::from_binary(&engine, &GUEST_RS_WASI_MODULE).unwrap();
    let linker = Linker::new(&engine);

    bids.map(ctx, move |bid| {
        let mut store = Store::new(&engine, ());
        let instance = linker.instantiate(&mut store, &component).unwrap();
        let intf_export = instance
            .get_export(&mut store, None, "pkg:component/nexmark")
            .unwrap();
        let func_print_export = instance
            .get_export(&mut store, Some(&intf_export), "q1")
            .unwrap();
        let func_print_typed = instance
            .get_typed_func::<(u64, u64, u64, u64), ((u64, u64, u64, u64),)>(&mut store, func_print_export)
            .unwrap();

        let ((auction, price, bidder, date_time),) =
            func_print_typed.call(&mut store, (bid.auction, bid.price, bid.bidder, bid.date_time))
            .unwrap();
        Output::new(auction, price, bidder, date_time)
    })
    .drain(ctx);
}