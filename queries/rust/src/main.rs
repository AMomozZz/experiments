pub mod data;
pub mod wasm;
pub mod either;

pub mod q1;
pub mod q2;
pub mod q3;
pub mod q4;
pub mod q5;
pub mod q6;
pub mod q7;
pub mod q8;
pub mod qw;
pub mod qs;

use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

use ::csv::ReaderBuilder;
use data::CompareOpV;
use data::Q6JoinOutput;
use data::Q7PrunedBid;
use data::QwOutput;
use data::QwPrunedBid;
use either::EitherData;
use runtime::prelude::stream::Event;
use crate::wasm::WasmComponent;
use runtime::prelude::serde::de::DeserializeOwned;
use runtime::prelude::*;
use runtime::traits::Timestamp;
use wasm::Host;
use wasm::WasmFunction;

use crate::data::Auction;
use crate::data::Bid;
use crate::data::Person;
use crate::data::{Q4PrunedAuction, Q4PrunedBid, Q5PrunedBid};
use wasmtime::{component::Linker, Config, Engine, Store};
use wasmtime_wasi::WasiImpl;

const USAGE: &str = "Usage: cargo run <data-dir> <query-id>";

const WATERMARK_FREQUENCY: usize = 1000;
const SLACK: Duration = Duration::from_milliseconds(100);

const GUEST_RS_WASI_MODULE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../guest-rs/target/wasm32-wasip2/release/component.wasm"
));

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
    let components_bids = std::fs::File::open(&format!("{dir}/component_bids.csv")).map(iter::<WasmComponent>);

    let config = Config::new();
    // config.async_support(true);
    // let config = config.cache_config_load("D:/master/thesis/aqualang/webassembly/experiments/queries/rust/Cargo.toml").unwrap();
    // config.enable_incremental_compilation();
    let engine = Engine::new(&config).unwrap();
    let host = Host::new();

    let wi: WasiImpl<Host> = WasiImpl(wasmtime_wasi::IoImpl::<Host>(host));
    let store_wrapper = Rc::new(RefCell::new(Store::new(&engine, wi)));

    let mut linker= Linker::new(&engine);
    // linker.root().;
    wasmtime_wasi::add_to_linker_sync::<WasiImpl<Host>>(&mut linker).unwrap();
    // let component = Component::from_binary(&engine, &GUEST_RS_WASI_MODULE).unwrap();

    let wasm_func_u64_compare_lt_m = WasmFunction::<(Vec<(u64, u64)>,), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/u64-compare", "lt-m");

    let wasm_func_q1 = WasmFunction::<(u64, u64, u64, u64), ((u64, u64, u64, u64),)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "q1");
    // let wasm_func_q1 = WasmFunction::<(Bid,), (Bid,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "q1");
    let wasm_func_q2 = WasmFunction::<(u64, u64, Vec<u64>,), (Option<(u64, u64)>,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "q2");
    let wasm_func_single_filter = WasmFunction::<(u64, Vec<u64>, ), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "single-filter");
    let wasm_func_multi_filter = WasmFunction::<(Vec<(u64, Vec<u64>)>, ), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "multi-filter");
    let wasm_func_multi_filter_opt = WasmFunction::<(Vec<(u64, Vec<u64>)>, ), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "multi-filter-opt");
    let wasm_func_string_sf = WasmFunction::<(String, Vec<String>, ), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "string-single-filter");
    // let wasm_func_less_equal_s = WasmFunction::<(u64,u64, ), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "less-or-equal-single");
    let wasm_func_less_equal_m = WasmFunction::<(Vec<(u64,u64)>, ), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "less-or-equal-multi");
    let wasm_func_q4_max_of_bid_price = WasmFunction::<(Vec<(Auction, Bid)>, ), (u64,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "q4-max-of-bid-price");
    let wasm_func_q4_max_of_bid_price_p = WasmFunction::<(Vec<(Q4PrunedAuction, Q4PrunedBid)>, ), (u64,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "q4-max-of-bid-price-p");
    let wasm_func_q4_avg_p = WasmFunction::<(Vec<(u64, u64)>, ), (u64,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "q4-avg");
    let wasm_func_q5_count = WasmFunction::<(Vec<Q5PrunedBid>, ), (u64,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "q5-count");
    let wasm_func_q5_max_by_key = WasmFunction::<(Vec<(u64, u64)>, ), (u64,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "q5-max-by-key");
    let wasm_func_q6_multi_compare = WasmFunction::<(Vec<CompareOpV>, ), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "q6-multi-comparison-v");
    let wasm_func_q6_avg = WasmFunction::<(Vec<Q6JoinOutput>, ), (u64,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "q6-avg");
    let wasm_func_q7 = WasmFunction::<(Vec<Q7PrunedBid>, ), (Q7PrunedBid,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "q7");
    let wasm_func_qw = WasmFunction::<(Vec<QwPrunedBid>, ), (QwOutput,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "qw");


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
        "qs-wasm" => {
            let empty_wasm_func = WasmFunction::new_empty(&linker, &engine, &store_wrapper);
            timed(move |ctx| qs::run_wasm_operator(stream(ctx, bids), stream_with(ctx, components_bids, 1), ctx, empty_wasm_func))
        },
        "qs-wasm-g" => {
            let empty_wasm_func = WasmFunction::new_empty(&linker, &engine, &store_wrapper);
            timed(move |ctx| qs::run_wasm_operator_g(stream(ctx, bids).map(ctx, |data| EitherData::Bid(data)), stream_with(ctx, components_bids, 1), ctx, empty_wasm_func))
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
                if components_bids.is_ok() {
                    stream_with(ctx, components_bids, 1).drain(ctx);
                }
            });
        },

        // map
        "io-with-map" => timed(move |ctx| {
            if bids.is_ok() {
                stream(ctx, bids).map(ctx, |data| EitherData::Bid(data)).drain(ctx);
            }
            if persons.is_ok() {
                stream(ctx, persons).map(ctx, |data| EitherData::Person(data)).drain(ctx);
            }
            if auctions.is_ok() {
                stream(ctx, auctions).map(ctx, |data| EitherData::Auction(data)).drain(ctx);
            }
            if components_bids.is_ok() {
                stream_with(ctx, components_bids, 1).drain(ctx);
            }
        }),

        "io-datas" => {
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

        "io-datas-with-map" => timed(move |ctx| {
            if bids.is_ok() {
                stream(ctx, bids).map(ctx, |data| EitherData::Bid(data)).drain(ctx);
            }
            if persons.is_ok() {
                stream(ctx, persons).map(ctx, |data| EitherData::Person(data)).drain(ctx);
            }
            if auctions.is_ok() {
                stream(ctx, auctions).map(ctx, |data| EitherData::Auction(data)).drain(ctx);
            }
        }),

        "io-components" => timed(move |ctx| {
            if components_bids.is_ok() {
                stream_with(ctx, components_bids, 1).drain(ctx);
            }
        }),

        //load and switch component
        "switch-component" => {
            let mut empty_wasm_func: WasmFunction<(EitherData,), (Option<EitherData>,)> = WasmFunction::new_empty(&linker, &engine, &store_wrapper);
            timed(move |ctx| {
                if components_bids.is_ok() {
                    let mut s: Stream<WasmComponent> = stream_with(ctx, components_bids, 1);
                    ctx.operator(move |_: stream::Collector<Option<Bid>>| async move {
                        loop {
                            match s.recv().await {
                                Event::Data(_, ref either) => {
                                    empty_wasm_func.switch(&either.file, &either.pkg_name, &either.name);
                                },
                                Event::Sentinel => break,
                                _ => {},
                            }
                        }
                        Ok(())
                    }).drain(ctx)
                }
            })
        },

        _ => panic!("unknown query"),
    }
}

// Buffered CSV reader
fn iter<T: Data + DeserializeOwned + 'static>(file: File) -> impl Iterator<Item = T> {
    let reader = BufReader::new(file);
    let csv_reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(reader);

    csv_reader
        .into_deserialize::<T>() 
        .map(move |result| match result {
            Ok(data) => {
                data
            },
            Err(e) => {
                panic!("CSV deserialization failed: {:?}", e);
            }
        })
}

// Stream from iterator
fn stream_with<T: Data + Timestamp>(
    ctx: &mut Context,
    iter: std::io::Result<impl Iterator<Item = T> + Send + 'static>,
    frequency: usize,
) -> Stream<T> {
    Stream::from_iter(ctx, iter.unwrap(), T::timestamp, frequency, SLACK)
}

fn stream<T: Data + Timestamp>(
    ctx: &mut Context,
    iter: std::io::Result<impl Iterator<Item = T> + Send + 'static>,
) -> Stream<T> {
    stream_with(ctx, iter, WATERMARK_FREQUENCY)
}
