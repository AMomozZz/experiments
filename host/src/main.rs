pub mod data;
pub mod wasm;
pub mod e1;

use std::{cell::RefCell, fs::File, io::BufReader, rc::Rc};
use csv::ReaderBuilder;
use runtime::{traits::{Timestamp, Data}, prelude::{Context, CurrentThreadRunner, Duration, Stream, serde::de::DeserializeOwned}};
use data::Bid;
use wasm::{Host, WasmFunction};
use wasmtime::{component::Linker, Config, Engine, Store};
use wasmtime_wasi::WasiImpl;

const GUEST_RS_WASI_MODULE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "./../guest/target/wasm32-wasip2/release/component.wasm"
));

const DATA_DIR: &str = "nexmark-data/bid";

const WATERMARK_FREQUENCY: usize = 1000;
const SLACK: Duration = Duration::from_milliseconds(100);

fn main() {
    let config = Config::new();
    let engine = Engine::new(&config).unwrap();
    let host = Host::new();
    let wi: WasiImpl<Host> = WasiImpl(wasmtime_wasi::IoImpl::<Host>(host));
    let store_wrapper = Rc::new(RefCell::new(Store::new(&engine, wi)));
    let mut linker= Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync::<WasiImpl<Host>>(&mut linker).unwrap();

    let wasm_func_q2 = WasmFunction::<(u64, u64, Vec<u64>,), (Option<(u64, u64)>,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "q2");
    let wasm_func_single_filter = WasmFunction::<(u64, Vec<u64>, ), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "single-filter");
    let wasm_func_e1 = WasmFunction::<(u64,), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "e1");

    let io_bids = std::fs::File::open(&format!("{DATA_DIR}/bids.csv")).map(iter::<Bid>);
    let native_bids = std::fs::File::open(&format!("{DATA_DIR}/bids.csv")).map(iter::<Bid>);
    let wasm_bids = std::fs::File::open(&format!("{DATA_DIR}/bids.csv")).map(iter::<Bid>);
    let wasm_opt_bids = std::fs::File::open(&format!("{DATA_DIR}/bids.csv")).map(iter::<Bid>);
    let wasm_opt_bids2 = std::fs::File::open(&format!("{DATA_DIR}/bids.csv")).map(iter::<Bid>);

    
    println!("Running io...");
    timed(move |ctx| stream(ctx, io_bids).drain(ctx));

    println!("Running native opt...");
    timed(move |ctx| e1::run_opt(stream(ctx, native_bids), ctx));

    println!("Running wasm (pass all data)...");
    timed(move |ctx| e1::run_wasm(stream(ctx, wasm_bids), ctx, wasm_func_q2));
    
    println!("Running wasm opt (pruned data)...");
    timed(move |ctx| e1::run_wasm_sf(stream(ctx, wasm_opt_bids), ctx, wasm_func_single_filter));

    println!("Running wasm opt2 (pruned data + filter conditions in wasm)...");
    timed(move |ctx| e1::run_wasm_e1(stream(ctx, wasm_opt_bids2), ctx, wasm_func_e1));
    
    // print_comparison(&rust_result, &wasm_result, &optimized_result);
}

fn timed(f: impl FnOnce(&mut Context) + Send + 'static) {
    let time = std::time::Instant::now();
    CurrentThreadRunner::run(f);
    eprintln!("{}", time.elapsed().as_millis());
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

// pub struct ExperimentResult {
//     pub name: String,
//     pub duration: Duration,
//     pub events_count: usize,
//     pub matched_count: usize,
//     pub throughput: f64,
// }

// impl ExperimentResult {
//     pub fn print(&self) {
//         println!("Experiment: {}", self.name);
//         println!("  Processed {} events in {:?}", self.events_count, self.duration);
//         println!("  Found {} matching bids", self.matched_count);
//         println!("  Throughput: {:.2} events/sec", self.throughput);
//         println!();
//     }
// }

// fn print_comparison(
//     rust_result: &ExperimentResult,
//     wasm_result: &ExperimentResult,
//     optimized_result: &ExperimentResult
// ) {
//     println!("=== Performance Comparison ===");
    
//     println!("Baseline (Rust Native): {:.2} events/sec", rust_result.throughput);
    
//     let wasm_relative = wasm_result.throughput / rust_result.throughput;
//     println!(
//         "WebAssembly UDF: {:.2} events/sec ({:.2}x of baseline)", 
//         wasm_result.throughput, 
//         wasm_relative
//     );
    
//     let optimized_relative = optimized_result.throughput / rust_result.throughput;
//     let optimized_vs_basic = optimized_result.throughput / wasm_result.throughput;
//     println!(
//         "Optimized WebAssembly UDF: {:.2} events/sec ({:.2}x of baseline, {:.2}x of basic WASM)", 
//         optimized_result.throughput, 
//         optimized_relative,
//         optimized_vs_basic
//     );
// }