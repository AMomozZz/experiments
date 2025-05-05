pub mod data;
pub mod wasm;
pub mod e1;

use std::{cell::RefCell, fs::File, io::BufReader, rc::Rc};
use csv::ReaderBuilder;
use runtime::{prelude::{serde::de::DeserializeOwned, Context, CurrentThreadRunner, Duration, Stream}, traits::{Data, Timestamp}};
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

    println!("Running io...");
    let mut io = ExperimentResult::new("io", 5);
    for _ in 0..15 {
        let bids = std::fs::File::open(&format!("{DATA_DIR}/bids.csv")).map(iter::<Bid>);
        let r = timed(move |ctx| stream(ctx, bids).drain(ctx));
        io.add(r);
    }
    io.print();

    println!("Running native opt...");
    let mut n_opt = ExperimentResult::new("io", 5);
    for _ in 0..15 {
        let bids = std::fs::File::open(&format!("{DATA_DIR}/bids.csv")).map(iter::<Bid>);
        let r = timed(move |ctx| e1::run_opt(stream(ctx, bids), ctx));
        n_opt.add(r);
    }
    n_opt.print();

    println!("Running wasm (pass all data)...");
    let mut wasm = ExperimentResult::new("io", 5);
    for _ in 0..15 {
        let bids = std::fs::File::open(&format!("{DATA_DIR}/bids.csv")).map(iter::<Bid>);
        let wasm_func_q2 = WasmFunction::<(u64, u64, Vec<u64>,), (Option<(u64, u64)>,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "q2");
        let r = timed(move |ctx| e1::run_wasm(stream(ctx, bids), ctx, wasm_func_q2));
        wasm.add(r);
    }
    wasm.print();
    
    println!("Running wasm opt (pruned data)...");
    let mut wasm_opt = ExperimentResult::new("io", 5);
    for _ in 0..15 {
        let bids = std::fs::File::open(&format!("{DATA_DIR}/bids.csv")).map(iter::<Bid>);
        let wasm_func_single_filter = WasmFunction::<(u64, Vec<u64>, ), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "single-filter");
        let r = timed(move |ctx| e1::run_wasm_sf(stream(ctx, bids), ctx, wasm_func_single_filter));
        wasm_opt.add(r);
    }
    wasm_opt.print();

    println!("Running wasm opt2 (pruned data + filter conditions in wasm)...");
    let mut wasm_opt2 = ExperimentResult::new("io", 5);
    for _ in 0..15 {
        let bids = std::fs::File::open(&format!("{DATA_DIR}/bids.csv")).map(iter::<Bid>);
        let wasm_func_e1 = WasmFunction::<(u64,), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, &store_wrapper, "pkg:component/nexmark", "e1");
        let r = timed(move |ctx| e1::run_wasm_e1(stream(ctx, bids), ctx, wasm_func_e1));
        wasm_opt2.add(r);
    }
    wasm_opt2.print();
    
    // print_comparison(&rust_result, &wasm_result, &optimized_result);
}

fn timed(f: impl FnOnce(&mut Context) + Send + 'static) -> u128 {
    let start = std::time::Instant::now();
    CurrentThreadRunner::run(f);
    let stop = start.elapsed().as_millis();
    // eprintln!("{}", stop);
    stop
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

pub struct ExperimentResult {
    name: String,
    warmup: u128,
    durations: Vec<u128>,
}

impl ExperimentResult {
    pub fn new(name: &str, warmup: u128) -> Self {
        ExperimentResult { name: name.to_string(), warmup, durations: vec![]}
    }

    pub fn add(&mut self, r: u128) {
        self.durations.push(r);
    }

    pub fn print(&self) {
        let total_count = self.durations.len() as u128;
        let avg_need = self.durations.len() as u128 - self.warmup;
        println!("Experiment: {}", self.name);
        println!("  All: {:?}", self.durations);
        println!("  Processed {} events in {}", total_count, self.durations.iter().sum::<u128>());
        println!("  All {} executions took an average of {} milliseconds", total_count, self.durations.iter().sum::<u128>() / total_count);
        println!("  The last {} executions took an average of {} milliseconds", avg_need, self.durations.iter().rev().take(avg_need as usize).sum::<u128>() / avg_need);
        println!();
    }
}

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