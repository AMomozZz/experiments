pub mod data;

pub mod q1;
pub mod q2;
pub mod q3;
pub mod q4;
pub mod q5;
pub mod q6;
pub mod q7;
pub mod q8;
pub mod qw;

use std::cell::RefCell;
use std::fs::File;
use std::io::BufRead;
use std::rc::Rc;

use data::CompareOpV;
use data::Q6JoinOutput;
use data::Q7PrunedBid;
use data::QwOutput;
use runtime::prelude::formats::csv;
use runtime::prelude::*;
use runtime::traits::Timestamp;
use wasmtime::component::TypedFunc;

use crate::data::Auction;
use crate::data::Bid;
use crate::data::Person;
use crate::data::{Q4PrunedAuction, Q4PrunedBid, Q5PrunedBid};
use wasmtime::{component::{Component, Linker, ResourceTable}, Config, Engine, Store};
use wasmtime_wasi::WasiImpl;

const USAGE: &str = "Usage: cargo run <data-dir> <query-id>";

const WATERMARK_FREQUENCY: usize = 1000;
const SLACK: Duration = Duration::from_milliseconds(100);

const GUEST_RS_WASI_MODULE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../guest-rs/target/wasm32-wasip2/release/component.wasm"
));

// host
pub struct Host {
    ctx: wasmtime_wasi::WasiCtx,
    table: ResourceTable,
}

impl wasmtime_wasi::WasiView for Host {
    fn ctx(&mut self) -> &mut wasmtime_wasi::WasiCtx {
        &mut self.ctx
    }
}

impl wasmtime_wasi::IoView for Host {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl Host {
    fn new() -> Self {
        let ctx = wasmtime_wasi::WasiCtxBuilder::new().inherit_stdio().build();
        let table = ResourceTable::new();
        Self { ctx, table }
    }
}

#[derive(Clone, Send, Sync)]
pub struct WasmFunction<I, O> {
    store: Rc<RefCell<Store<WasiImpl<Host>>>>,
    func: TypedFunc<I, O>
}

impl<I, O> WasmFunction<I, O> 
where 
    I: wasmtime::component::Lower + wasmtime::component::ComponentNamedList,
    O: wasmtime::component::Lift + wasmtime::component::ComponentNamedList
{
    fn new(func: TypedFunc<I, O>, store_wrapper: Rc<RefCell<Store<WasiImpl<Host>>>>) -> Self {
        WasmFunction {
            store: store_wrapper,
            func
        }
    }
    fn call(&self, input: I) -> O {
        let result = self.func.call(&mut *self.store.borrow_mut(), input).unwrap();
        self.func.post_return(&mut *self.store.borrow_mut()).unwrap();
        result
    }
}

fn get_wasm_func<I: wasmtime::component::Lower + wasmtime::component::ComponentNamedList, O: wasmtime::component::Lift + wasmtime::component::ComponentNamedList>(linker: &Linker<WasiImpl<Host>>, component: &Component, store_wrapper: &Rc<RefCell<Store<WasiImpl<Host>>>>, pkg_name: &str, name: &str) -> WasmFunction<I, O> {
    let clone_store_wrapper = store_wrapper.clone();
    WasmFunction::<I, O>::new(get_func_from_component(linker, component, &clone_store_wrapper, pkg_name, name), clone_store_wrapper)
}

fn get_func_from_component<I: wasmtime::component::Lower + wasmtime::component::ComponentNamedList, O: wasmtime::component::Lift + wasmtime::component::ComponentNamedList>(linker: &Linker<WasiImpl<Host>>, component: &Component, store_wrapper: &Rc<RefCell<Store<WasiImpl<Host>>>>, pkg_name: &str, name: &str) -> wasmtime::component::TypedFunc<I, O> {
    let mut store = store_wrapper.borrow_mut();
    let instance = linker.instantiate(&mut *store, component).unwrap();
    let intf_export = instance
        .get_export(&mut *store, None, pkg_name)
        .unwrap();
    let func_export = instance
        .get_export(&mut *store, Some(&intf_export), name)
        .unwrap();
    instance
        .get_typed_func::<I, O>(&mut *store, func_export)
        .unwrap()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(dir) = args.next() else {
        println!("{USAGE}");
        return;
    };
    let Some(query) = args.next() else {
        println!("{USAGE}");
        return;
    };

    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
    let auctions = std::fs::File::open(&format!("{dir}/auctions.csv")).map(iter::<Auction>);
    let persons = std::fs::File::open(&format!("{dir}/persons.csv")).map(iter::<Person>);

    let config = Config::new();
    // config.async_support(true);
    // config.cache_config_load();
    // config.enable_incremental_compilation();
    let engine = Engine::new(&config).unwrap();
    let host = Host::new();

    let wi: WasiImpl<Host> = WasiImpl(wasmtime_wasi::IoImpl::<Host>(host));
    let store_wrapper = Rc::new(RefCell::new(Store::new(&engine, wi)));

    let mut linker= Linker::new(&engine);
    // linker.root().;
    let component = Component::from_binary(&engine, &GUEST_RS_WASI_MODULE).unwrap();
    wasmtime_wasi::add_to_linker_sync::<WasiImpl<Host>>(&mut linker).unwrap();

    let wasm_func_u64_compare_lt_m = get_wasm_func::<(Vec<(u64, u64)>,), (bool,)>(&linker, &component, &store_wrapper, "pkg:component/u64-compare", "lt-m");

    let wasm_func_q1 = get_wasm_func::<(u64, u64, u64, u64), ((u64, u64, u64, u64),)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "q1");
    // let wasm_func_q1 = get_wasm_func::<(Bid,), (Bid,)>(&linker, &component, &store_wrapper, "q1");
    let wasm_func_q2 = get_wasm_func::<(u64, u64, Vec<u64>,), (Option<(u64, u64)>,)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "q2");
    let wasm_func_single_filter = get_wasm_func::<(u64, Vec<u64>, ), (bool,)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "single-filter");
    let wasm_func_multi_filter = get_wasm_func::<(Vec<(u64, Vec<u64>)>, ), (bool,)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "multi-filter");
    let wasm_func_multi_filter_opt = get_wasm_func::<(Vec<(u64, Vec<u64>)>, ), (bool,)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "multi-filter-opt");
    let wasm_func_string_sf = get_wasm_func::<(String, Vec<String>, ), (bool,)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "string-single-filter");
    // let wasm_func_less_equal_s = get_wasm_func::<(u64,u64, ), (bool,)>(&linker, &component, &store_wrapper, "less-or-equal-single");
    let wasm_func_less_equal_m = get_wasm_func::<(Vec<(u64,u64)>, ), (bool,)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "less-or-equal-multi");
    let wasm_func_q4_max_of_bid_price = get_wasm_func::<(Vec<(Auction, Bid)>, ), (u64,)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "q4-max-of-bid-price");
    let wasm_func_q4_max_of_bid_price_p = get_wasm_func::<(Vec<(Q4PrunedAuction, Q4PrunedBid)>, ), (u64,)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "q4-max-of-bid-price-p");
    let wasm_func_q4_avg_p = get_wasm_func::<(Vec<(u64, u64)>, ), (u64,)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "q4-avg");
    let wasm_func_q5_count = get_wasm_func::<(Vec<Q5PrunedBid>, ), (u64,)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "q5-count");
    let wasm_func_q5_max_by_key = get_wasm_func::<(Vec<(u64, u64)>, ), (u64,)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "q5-max-by-key");
    let wasm_func_q6_multi_compare = get_wasm_func::<(Vec<CompareOpV>, ), (bool,)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "q6-multi-comparison-v");
    let wasm_func_q6_avg = get_wasm_func::<(Vec<Q6JoinOutput>, ), (u64,)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "q6-avg");
    let wasm_func_q7 = get_wasm_func::<(Vec<Q7PrunedBid>, ), (Q7PrunedBid,)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "q7");
    let wasm_func_qw = get_wasm_func::<(Vec<Bid>, ), (QwOutput,)>(&linker, &component, &store_wrapper, "pkg:component/nexmark", "qw");


    fn timed(f: impl FnOnce(&mut Context) + Send + 'static) {
        let time = std::time::Instant::now();
        CurrentThreadRunner::run(f);
        eprintln!("{}", time.elapsed().as_millis());
    }

    match query.as_str() {
        // Un-optimised
        "q1" => timed(move |ctx| q1::run(stream(ctx, bids), ctx)),
        "q2" => timed(move |ctx| q2::run(stream(ctx, bids), ctx)),
        "q3" => timed(move |ctx| q3::run(stream(ctx, auctions), stream(ctx, persons), ctx)),
        "q4" => timed(move |ctx| q4::run(stream(ctx, auctions), stream(ctx, bids), ctx)),
        "q5" => timed(move |ctx| q5::run(stream(ctx, bids), ctx)),
        "q6" => timed(move |ctx| q6::run(stream(ctx, auctions), stream(ctx, bids), ctx)),
        "q7" => timed(move |ctx| q7::run(stream(ctx, bids), ctx)),
        "q8" => timed(move |ctx| q8::run(stream(ctx, auctions), stream(ctx, persons), ctx)),
        "qw" => {
            let Some(size) = args.next() else {
                println!("{USAGE} <size> <step>");
                return;
            };
            let Some(step) = args.next() else {
                println!("{USAGE} <size> <step>");
                return;
            };
            let size = size.parse().unwrap();
            let step = step.parse().unwrap();
            timed(move |ctx| qw::run(stream(ctx, bids), size, step, ctx))
        },
        // Optimised
        "q1-opt" => timed(move |ctx| q1::run_opt(stream(ctx, bids), ctx)),
        "q2-opt" => timed(move |ctx| q2::run_opt(stream(ctx, bids), ctx)),
        "q3-opt" => timed(move |ctx| q3::run_opt(stream(ctx, auctions), stream(ctx, persons), ctx)),
        "q4-opt" => timed(move |ctx| q4::run_opt(stream(ctx, auctions), stream(ctx, bids), ctx)),
        "q5-opt" => timed(move |ctx| q5::run_opt(stream(ctx, bids), ctx)),
        "q6-opt" => timed(move |ctx| q6::run_opt(stream(ctx, auctions), stream(ctx, bids), ctx)),
        "q7-opt" => timed(move |ctx| q7::run_opt(stream(ctx, bids), ctx)),
        "q8-opt" => timed(move |ctx| q8::run_opt(stream(ctx, auctions), stream(ctx, persons), ctx)),
        "qw-opt" => {
            let size = args.next().unwrap().parse().unwrap();
            let step = args.next().unwrap().parse().unwrap();
            timed(move |ctx| qw::run_opt(stream(ctx, bids), size, step, ctx))
        },
        // wasm
        "q1-wasm" => timed(move |ctx| q1::run_wasm(stream(ctx, bids), ctx, wasm_func_q1)),
        "q2-wasm" => timed(move |ctx| q2::run_wasm(stream(ctx, bids), ctx, wasm_func_q2)),
        "q2-wasm-sf" => timed(move |ctx| q2::run_wasm_sf(stream(ctx, bids), ctx, wasm_func_single_filter)),
        "q2-wasm-mf" => timed(move |ctx| q2::run_wasm_mf(stream(ctx, bids), ctx, wasm_func_multi_filter)),
        "q2-wasm-mf-opt" => timed(move |ctx| q2::run_wasm_mf(stream(ctx, bids), ctx, wasm_func_multi_filter_opt)),
        "q3-wasm" => timed(move |ctx| q3::run_wasm(stream(ctx, auctions), stream(ctx, persons), ctx, wasm_func_string_sf, wasm_func_single_filter)),
        "q4-wasm" => timed(move |ctx| q4::run_wasm(stream(ctx, auctions), stream(ctx, bids), ctx, wasm_func_less_equal_m, wasm_func_q4_max_of_bid_price, wasm_func_q4_avg_p)),
        "q4-wasm-p" => timed(move |ctx| q4::run_wasm_p(stream(ctx, auctions), stream(ctx, bids), ctx, wasm_func_less_equal_m, wasm_func_q4_max_of_bid_price_p, wasm_func_q4_avg_p)),
        "q5-wasm" => timed(move |ctx| q5::run_wasm(stream(ctx, bids), ctx, wasm_func_q5_count, wasm_func_q5_max_by_key)),
        "q6-wasm" => timed(move |ctx| q6::run_wasm(stream(ctx, auctions), stream(ctx, bids), ctx, wasm_func_q6_multi_compare, wasm_func_q6_avg)),
        "q6-wasm-ng" => timed(move |ctx| q6::run_wasm_ng(stream(ctx, auctions), stream(ctx, bids), ctx, wasm_func_u64_compare_lt_m, wasm_func_q6_avg)),
        "q7-wasm" => timed(move |ctx| q7::run_wasm(stream(ctx, bids), ctx, wasm_func_q7)),
        "qw-wasm" => {
            let size = args.next().unwrap().parse().unwrap();
            let step = args.next().unwrap().parse().unwrap();
            timed(move |ctx| qw::run_wasm(stream(ctx, bids), size, step, ctx, wasm_func_qw))
        },

        // io
        "io" => {
            timed(move |ctx| {
                if bids.is_ok() {
                    stream(ctx, bids).drain(ctx);
                }
                if persons.is_ok() {
                    stream(ctx, persons).drain(ctx);
                }
                if auctions.is_ok() {
                    stream(ctx, auctions).drain(ctx);
                }
            });
        },
        _ => panic!("unknown query"),
    }
}

// Buffered CSV reader
fn iter<T: Data>(file: File) -> impl Iterator<Item = T> {
    let mut bufreader = std::io::BufReader::new(file);
    let mut buf = std::vec::Vec::new();
    let mut reader = csv::de::Reader::<1024>::new(',');
    std::iter::from_fn(move || match bufreader.read_until(b'\n', &mut buf) {
        Ok(0) => None,
        Ok(n) => {
            let mut de = csv::de::Deserializer::new(&mut reader, &buf[0..n]);
            match T::deserialize(&mut de) {
                Ok(data) => {
                    buf.clear();
                    Some(data)
                }
                Err(e) => panic!("Failed to deserialize: {}", e),
            }
        }
        Err(e) => panic!("Failed to read from stdin: {}", e),
    })
}

// Stream from iterator
fn stream<T: Data + Timestamp>(
    ctx: &mut Context,
    iter: std::io::Result<impl Iterator<Item = T> + Send + 'static>,
) -> Stream<T> {
    Stream::from_iter(ctx, iter.unwrap(), T::timestamp, WATERMARK_FREQUENCY, SLACK)
}
