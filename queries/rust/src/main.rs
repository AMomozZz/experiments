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

use runtime::prelude::formats::csv;
use runtime::prelude::*;
use runtime::traits::Timestamp;

use crate::data::Auction;
use crate::data::Bid;
use crate::data::Person;
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

// pub struct StoreWrapper(RefCell<Store<WasiImpl<Host>>>);

// unsafe impl Send for StoreWrapper {}
// unsafe impl Sync for StoreWrapper {}

fn get_func_from_component<I: wasmtime::component::Lower + wasmtime::component::ComponentNamedList, O: wasmtime::component::Lift + wasmtime::component::ComponentNamedList>(linker: &Linker<WasiImpl<Host>>, component: &Component, store_wrapper: &RefCell<Store<WasiImpl<Host>>>, name: &str) -> wasmtime::component::TypedFunc<I, O> {
    let mut store = store_wrapper.borrow_mut();
    let instance = linker.instantiate(&mut *store, component).unwrap();
    let intf_export = instance
        .get_export(&mut *store, None, "pkg:component/nexmark")
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
    let engine = Engine::new(&config).unwrap();
    let host = Host::new();

    let wi: WasiImpl<Host> = WasiImpl(wasmtime_wasi::IoImpl::<Host>(host));
    let store_wrapper = RefCell::new(Store::new(&engine, wi));

    let mut linker= Linker::new(&engine);
    let component = Component::from_binary(&engine, &GUEST_RS_WASI_MODULE).unwrap();
    wasmtime_wasi::add_to_linker_sync::<WasiImpl<Host>>(&mut linker).unwrap();

    let func_q1_typed = get_func_from_component::<(u64, u64, u64, u64), ((u64, u64, u64, u64),)>(&linker, &component, &store_wrapper, "q1");
    let func_q2_typed = get_func_from_component::<(u64, u64, Vec<u64>,), (Option<(u64, u64)>,)>(&linker, &component, &store_wrapper, "q2");
    let func_single_filter_typed = get_func_from_component::<(u64, Vec<u64>, ), (bool,)>(&linker, &component, &store_wrapper, "single-filter");
    let func_multi_filter_typed = get_func_from_component::<(Vec<(u64, Vec<u64>)>, ), (bool,)>(&linker, &component, &store_wrapper, "multi-filter");
    let func_multi_filter_opt_typed = get_func_from_component::<(Vec<(u64, Vec<u64>)>, ), (bool,)>(&linker, &component, &store_wrapper, "multi-filter-opt");
    // let func_q3_typed = get_func_from_component(linker, &component, store_wrapper, "q3");

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
        "q1-wasm" => timed(move |ctx| q1::run_wasm(stream(ctx, bids), ctx, func_q1_typed, store_wrapper)),
        "q2-wasm" => timed(move |ctx| q2::run_wasm(stream(ctx, bids), ctx, func_q2_typed, store_wrapper)),
        "q2-wasm-sf" => timed(move |ctx| q2::run_wasm_sf(stream(ctx, bids), ctx, func_single_filter_typed, store_wrapper)),
        "q2-wasm-mf" => timed(move |ctx| q2::run_wasm_mf(stream(ctx, bids), ctx, func_multi_filter_typed, store_wrapper)),
        "q2-wasm-mf-opt" => timed(move |ctx| q2::run_wasm_mf(stream(ctx, bids), ctx, func_multi_filter_opt_typed, store_wrapper)),

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
