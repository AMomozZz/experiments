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

use std::fs::File;
use std::io::BufRead;
use std::sync::Arc;
use std::sync::Mutex;

use runtime::prelude::formats::csv;
use runtime::prelude::*;
use runtime::traits::Timestamp;

use crate::data::Auction;
use crate::data::Bid;
use crate::data::Person;
use wasmtime::{component::{Component, Linker, ResourceTable}, Config, Engine, Store};
use wasmtime_wasi::{Pollable, bindings::sockets::tcp_create_socket::TcpSocket, WasiImpl};

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

    let mut config = Config::new();
    // config.async_support(true);
    let engine = Engine::new(&config).unwrap();
    let host = Host::new();

    let wi: WasiImpl<Host> = WasiImpl(wasmtime_wasi::IoImpl::<Host>(host));
    let mut store = Store::new(&engine, wi);
    let mut linker: Linker<WasiImpl<Host>> = Linker::new(&engine);
    let component = Component::from_binary(&engine, &GUEST_RS_WASI_MODULE).unwrap();
    wasmtime_wasi::add_to_linker_sync::<WasiImpl<Host>>(&mut linker).unwrap();

    let instance = linker.instantiate(&mut store, &component).unwrap();
    let arc_store = Arc::new(Mutex::new(store));

    // let intf_export = instance
    //     .get_export(&mut *store, None, "pkg:component/nexmark")
    //     .unwrap();
    // let func_print_export = instance
    //     .get_export(&mut *store, Some(&intf_export), "q1")
    //     .unwrap();
    // let func_print_typed = instance
    //     .get_typed_func::<(u64, u64, u64, u64), ((u64, u64, u64, u64),)>(&mut *store, func_print_export)
    //     .unwrap();

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
        }
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
        }
        // wasm
        "q1-wasm" => timed(move |ctx| q1::run_wasm(stream(ctx, bids), ctx, instance, arc_store)),
        // "q1-wasm" => timed(move |ctx| q1::run_wasm(stream(ctx, bids), ctx, , arc_store)),
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
        }
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
