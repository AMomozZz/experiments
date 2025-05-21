pub mod data;
pub mod wasm;
pub mod either;
pub mod e1;
pub mod e2;
pub mod e3;

use std::{fs::{File, OpenOptions}, io::{BufReader, BufWriter}};
use csv::{ReaderBuilder, WriterBuilder};
use runtime::{prelude::{serde::de::DeserializeOwned, Context, CurrentThreadRunner, Duration, Stream}, traits::{Data, Timestamp}};
use data::{Bid, PrunedBid};
use wasm::{Host, WasmComponent, WasmFunction};
use wasmtime::{component::Linker, Config, Engine};
use wasmtime_wasi::WasiImpl;
use std::hint::black_box;
use chrono::Utc;

const USAGE: &str = "Usage: cargo run <data-dir> <experiment-id> <measure-experiment-num> <warmup-num> <output-dir>";

// const GUEST_RS_WASI_MODULE: &[u8] = include_bytes!(concat!(
//     env!("CARGO_MANIFEST_DIR"),
//     "./../guest/target/wasm32-wasip2/release/component.wasm"
// ));

const GUEST_RS_WASI_MODULE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "./../guest/target/wasm32-wasip2/release/component_usedonly.wasm"
));

// const DATA_DIR: &str = "nexmark-data/bid";
// const TOTAL: u128 = 15;
// const WARMUP: u128 = 5;

const WATERMARK_FREQUENCY: usize = 1000;
const SLACK: Duration = Duration::from_milliseconds(100);

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(dir) = args.next() else {
        println!("{USAGE}");
        return;
    };
    let Some(experiment) = args.next() else {
        println!("{USAGE}");
        return;
    };
    let Some(measure) = args.next() else {
        println!("{USAGE}");
        return;
    };
    let Ok(measure): Result<u128, _> = measure.parse() else {
        eprintln!("Invalid number: {measure}");
        return;
    };
    let Some(warmup) = args.next() else {
        println!("{USAGE}");
        return;
    };
    let Ok(warmup): Result<u128, _> = warmup.parse() else {
        eprintln!("Invalid number: {warmup}");
        return;
    };
    let Some(output_dir) = args.next() else {
        println!("{USAGE}");
        return;
    };

    let total: u128 = measure + warmup;
    let warmup: u128 = warmup;

    let config = Config::new();
    let engine = Engine::new(&config).unwrap();
    let mut linker= Linker::new(&engine);

    // let host = Host::new();
    // let wi: Rc<RefCell<WasiImpl<Host>>> = Rc::new(RefCell::new(WasiImpl(wasmtime_wasi::IoImpl::<Host>(host))));
    // let store_wrapper: Rc<RefCell<Store<WasiImpl<Host>>>> = Rc::new(RefCell::new(Store::new(&engine, wi)));
    
    wasmtime_wasi::add_to_linker_sync::<WasiImpl<Host>>(&mut linker).unwrap();

    match experiment.as_str() {
        "e1" => {
            let v = vec![100, 1000, 10000, 100000, 1000000];//, 10000000];

            for i in v {
                println!("Running e1 in loop {}", i);

                println!("Running io...");
                let mut io = ExperimentResult::new("io", warmup, &output_dir);
                for _ in 0..total {
                    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(|file| iter_with::<Bid>(file, i));
                    let r = timed(move |ctx| stream(ctx, bids).drain(ctx));
                    io.add(r);
                }
                io.print();
                io.in_file(experiment.as_str(), i);

                println!("Running native opt...");
                let mut n_opt = ExperimentResult::new("native opt", warmup, &output_dir);
                for _ in 0..total {
                    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(|file| iter_with::<Bid>(file, i));
                    let r = timed(move |ctx| e1::run_opt(stream(ctx, bids), ctx));
                    n_opt.add(r);
                }
                n_opt.print();
                n_opt.in_file(experiment.as_str(), i);

                println!("Running wasm (pass all data)...");
                let mut wasm = ExperimentResult::new("wasm (pass all data)", warmup, &output_dir);
                for _ in 0..total {
                    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(|file| iter_with::<Bid>(file, i));
                    let wasm_func_q2 = WasmFunction::<(u64, u64, Vec<u64>,), (Option<(u64, u64)>,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, "pkg:component/nexmark", "q2");
                    let r = timed(move |ctx| e1::run_wasm(stream(ctx, bids), ctx, wasm_func_q2));
                    wasm.add(r);
                }
                wasm.print();
                wasm.in_file(experiment.as_str(), i);
                
                println!("Running wasm opt (pruned data)...");
                let mut wasm_opt = ExperimentResult::new("wasm opt (pruned data)", warmup, &output_dir);
                for _ in 0..total {
                    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(|file| iter_with::<Bid>(file, i));
                    let wasm_func_single_filter = WasmFunction::<(u64, Vec<u64>, ), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, "pkg:component/nexmark", "single-filter");
                    let r = timed(move |ctx| e1::run_wasm_sf(stream(ctx, bids), ctx, wasm_func_single_filter));
                    wasm_opt.add(r);
                }
                wasm_opt.print();
                wasm_opt.in_file(experiment.as_str(), i);

                println!("Running wasm opt2 (pruned data + filter conditions in wasm)...");
                let mut wasm_opt2 = ExperimentResult::new("wasm opt2 (pruned data + filter conditions in wasm)", warmup, &output_dir);
                for _ in 0..total {
                    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(|file| iter_with::<Bid>(file, i));
                    let wasm_func_e1 = WasmFunction::<(u64,), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, "pkg:component/nexmark", "e1");
                    let r = timed(move |ctx| e1::run_wasm_e1(stream(ctx, bids), ctx, wasm_func_e1));
                    wasm_opt2.add(r);
                }
                wasm_opt2.print();
                wasm_opt2.in_file(experiment.as_str(), i);

                println!("Running wasm opt3 (structured data + filter conditions in wasm + directly returns a not pruned data)...");
                let mut wasm_opt3 = ExperimentResult::new("wasm opt3 (structured data + filter conditions in wasm + directly returns a not pruned data)", warmup, &output_dir);
                for _ in 0..total {
                    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(|file| iter_with::<Bid>(file, i));
                    let wasm_func_e1 = WasmFunction::<(Bid,), (Option<Bid>,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, "pkg:component/nexmark", "all-in-wasm-not-pruned");
                    let r = timed(move |ctx| e1::run_wasm_e1_all_in_wasm_g::<Bid>(stream(ctx, bids), ctx, wasm_func_e1));
                    wasm_opt3.add(r);
                }
                wasm_opt3.print();
                wasm_opt3.in_file(experiment.as_str(), i);

                println!("Running wasm opt4 (structured data + filter conditions in wasm + directly returns a pruned data)...");
                let mut wasm_opt4 = ExperimentResult::new("wasm opt4 (structured data + filter conditions in wasm + directly returns a pruned data)", warmup, &output_dir);
                for _ in 0..total {
                    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(|file| iter_with::<Bid>(file, i));
                    let wasm_func_e1 = WasmFunction::<(Bid,), (Option<PrunedBid>,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, "pkg:component/nexmark", "all-in-wasm");
                    let r = timed(move |ctx| e1::run_wasm_e1_all_in_wasm(stream(ctx, bids), ctx, wasm_func_e1));
                    wasm_opt4.add(r);
                }
                wasm_opt4.print();
                wasm_opt4.in_file(experiment.as_str(), i);
            }
        },

        "e2" => {
            let component_len = std::fs::File::open(&format!("{dir}/component_bids.csv")).map(iter::<WasmComponent>).unwrap().count();

            println!("Running io...");
            let mut io = ExperimentResult::new("io", warmup, &output_dir);
            for _ in 0..total {
                let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                let components_bids = std::fs::File::open(&format!("{dir}/component_bids.csv")).map(iter::<WasmComponent>);
                let r = timed(move |ctx| {
                    stream(ctx, bids).drain(ctx);
                    stream_with(ctx, components_bids, 1).drain(ctx);
                });
                io.add(r);
            }
            io.print();
            io.in_file(experiment.as_str(), component_len);

            println!("Running dynamic wasm opt2 filter...");
            let mut wasm_opt2 = ExperimentResult::new("wasm opt2 dynamic reload", warmup, &output_dir);
            for _ in 0..total {
                let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                let components_bids = std::fs::File::open(&format!("{dir}/component_bids.csv")).map(iter::<WasmComponent>);
                let wasm_func_e1 = WasmFunction::<(u64,), (bool,)>::new_empty_with_name(&linker, &engine, "pkg:component/nexmark", "e1");
                let r = timed(move |ctx| e2::run_wasm_e2(stream(ctx, bids), stream_with(ctx, components_bids, 1), ctx, wasm_func_e1));
                wasm_opt2.add(r);
            }
            wasm_opt2.print();
            wasm_opt2.in_file(experiment.as_str(), component_len);

            println!("Running dynamic wasm opt3 filter...");
            let mut wasm_opt3 = ExperimentResult::new("wasm opt3 dynamic reload", warmup, &output_dir);
            for _ in 0..total {
                let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                let components_bids = std::fs::File::open(&format!("{dir}/component_bids.csv")).map(iter::<WasmComponent>);
                let wasm_func_e1 = WasmFunction::<(Bid,), (Option<Bid>,)>::new_empty_with_name(&linker, &engine, "pkg:component/nexmark", "all-in-wasm-not-pruned");
                let r = timed(move |ctx| e2::run_wasm_operator_g(stream(ctx, bids), stream_with(ctx, components_bids, 1), ctx, wasm_func_e1));
                wasm_opt3.add(r);
            }
            wasm_opt3.print();
            wasm_opt3.in_file(experiment.as_str(), component_len);

            println!("Running dynamic wasm opt4 filter...");
            let mut wasm_opt4 = ExperimentResult::new("wasm opt4 dynamic reload", warmup, &output_dir);
            for _ in 0..total {
                let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                let components_bids = std::fs::File::open(&format!("{dir}/component_bids.csv")).map(iter::<WasmComponent>);
                let wasm_func_e1 = WasmFunction::<(Bid,), (Option<PrunedBid>,)>::new_empty_with_name(&linker, &engine, "pkg:component/nexmark", "all-in-wasm");
                let r = timed(move |ctx| e2::run_wasm_operator(stream(ctx, bids), stream_with(ctx, components_bids, 1), ctx, wasm_func_e1));
                wasm_opt4.add(r);
            }
            wasm_opt4.print();
            wasm_opt4.in_file(experiment.as_str(), component_len);
        },

        "e3" => {
            let v = vec![100, 1000, 10000, 100000, 1000000];//, 10000000];

            for i in v {
                println!("Running e3 in loop {}", i);

                println!("Running io...");
                let mut io = ExperimentResult::new("io", warmup, &output_dir);
                for _ in 0..total {
                    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                    let r = timed(move |_ctx| {
                        for bid in bids.unwrap().take(i) {
                            let _input = black_box(bid);
                        }
                    });
                    io.add(r);
                }
                io.print();
                io.in_file(experiment.as_str(), i);

                println!("Running native opt...");
                let mut n_opt = ExperimentResult::new("native opt", warmup, &output_dir);
                for _ in 0..total {
                    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                    let r = timed(move |_ctx| {
                        for bid in bids.unwrap().take(i) {
                            let input = black_box(bid);
                            let _output = black_box(e3::opt_func(input));
                        }
                    });
                    n_opt.add(r);
                }
                n_opt.print();
                n_opt.in_file(experiment.as_str(), i);

                println!("Running wasm (pass all data)...");
                let mut wasm = ExperimentResult::new("wasm (pass all data)", warmup, &output_dir);
                for _ in 0..total {
                    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                    let wasm_func_q2 = WasmFunction::<(u64, u64, Vec<u64>,), (Option<(u64, u64)>,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, "pkg:component/nexmark", "q2");
                    let r = timed(move |_ctx| {
                        for bid in bids.unwrap().take(i) {
                            let input = black_box(bid);
                            let _output = black_box(e3::run_wasm_func(input, |args| wasm_func_q2.call(args)));
                        }
                    });
                    wasm.add(r);
                }
                wasm.print();
                wasm.in_file(experiment.as_str(), i);
                
                println!("Running wasm opt (pruned data)...");
                let mut wasm_opt = ExperimentResult::new("wasm opt (pruned data)", warmup, &output_dir);
                for _ in 0..total {
                    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                    let wasm_func_single_filter = WasmFunction::<(u64, Vec<u64>, ), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, "pkg:component/nexmark", "single-filter");
                    let r = timed(move |_ctx| {
                        for bid in bids.unwrap().take(i) {
                            let input = black_box(bid);
                            let _output = black_box(e3::run_wasm_sf_func(input, |args| wasm_func_single_filter.call(args)));
                        }
                    });
                    wasm_opt.add(r);
                }
                wasm_opt.print();
                wasm_opt.in_file(experiment.as_str(), i);

                println!("Running wasm opt2 (pruned data + filter conditions in wasm)...");
                let mut wasm_opt2 = ExperimentResult::new("wasm opt2 (pruned data + filter conditions in wasm)", warmup, &output_dir);
                for _ in 0..total {
                    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                    let wasm_func_e1 = WasmFunction::<(u64,), (bool,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, "pkg:component/nexmark", "e1");
                    let r = timed(move |_ctx| {
                        for bid in bids.unwrap().take(i) {
                            let input = black_box(bid);
                            let _output = black_box(e3::run_wasm_e1_func(input, |args| wasm_func_e1.call(args)));
                        }
                    });
                    wasm_opt2.add(r);
                }
                wasm_opt2.print();
                wasm_opt2.in_file(experiment.as_str(), i);

                println!("Running wasm opt3 (structured data + filter conditions in wasm + directly returns a not pruned data)...");
                let mut wasm_opt3 = ExperimentResult::new("wasm opt3 (structured data + filter conditions in wasm + directly returns a not pruned data)", warmup, &output_dir);
                for _ in 0..total {
                    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                    let wasm_func_e1 = WasmFunction::<(Bid,), (Option<Bid>,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, "pkg:component/nexmark", "all-in-wasm-not-pruned");
                    let r = timed(move |_ctx| {
                        for bid in bids.unwrap().take(i) {
                            let input = black_box(bid);
                            let _output = black_box(wasm_func_e1.call((input.clone(),)).0);
                        }
                    });
                    wasm_opt3.add(r);
                }
                wasm_opt3.print();
                wasm_opt3.in_file(experiment.as_str(), i);

                println!("Running wasm opt4 (structured data + filter conditions in wasm + directly returns a pruned data)...");
                let mut wasm_opt4 = ExperimentResult::new("wasm opt4 (structured data + filter conditions in wasm + directly returns a pruned data)", warmup, &output_dir);
                for _ in 0..total {
                    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                    let wasm_func_e1 = WasmFunction::<(Bid,), (Option<PrunedBid>,)>::new(&linker, &engine, GUEST_RS_WASI_MODULE, "pkg:component/nexmark", "all-in-wasm");
                    let r = timed(move |_ctx| {
                        for bid in bids.unwrap().take(i) {
                            let input = black_box(bid);
                            let _output = black_box(wasm_func_e1.call((input.clone(),)).0);
                        }
                    });
                    wasm_opt4.add(r);
                }
                wasm_opt4.print();
                wasm_opt4.in_file(experiment.as_str(), i);
            }
        },

        "e4" => {
            let v = vec![1, 10, 100, 1000, 10000];//, 100000, 1000000, 10000000];

            for i in v {
                println!("Running e4 in loop {}", i);

                println!("Running io...");
                let mut io = ExperimentResult::new("io", warmup, &output_dir);
                for _ in 0..total {
                    let components_bids = std::fs::File::open(&format!("{dir}/component_bids.csv")).map(iter::<WasmComponent>);
                    let r = timed(move |_ctx| {
                        for components in components_bids.unwrap().take(i) {
                            let _input = black_box(components);
                        }
                    });
                    io.add(r);
                }
                io.print();
                io.in_file(experiment.as_str(), i);

                println!("Running dynamic wasm opt2 filter...");
                let mut wasm_opt2 = ExperimentResult::new("wasm opt2 dynamic reload", warmup, &output_dir);
                for _ in 0..total {
                    let components_bids = std::fs::File::open(&format!("{dir}/component_bids.csv")).map(iter::<WasmComponent>);
                    let mut wasm_func_e1 = WasmFunction::<(u64,), (bool,)>::new_empty_with_name(&linker, &engine, "pkg:component/nexmark", "e1");
                    let r = timed(move |_ctx| {
                        for components in components_bids.unwrap().take(i) {
                            let input = black_box(components);
                            let _output = black_box(wasm_func_e1.switch_default(&input.file));
                        }
                    });
                    wasm_opt2.add(r);
                }
                wasm_opt2.print();
                wasm_opt2.in_file(experiment.as_str(), i);

                println!("Running dynamic wasm opt3 filter...");
                let mut wasm_opt3 = ExperimentResult::new("wasm opt3 dynamic reload", warmup, &output_dir);
                for _ in 0..total {
                    let components_bids = std::fs::File::open(&format!("{dir}/component_bids.csv")).map(iter::<WasmComponent>);
                    let mut wasm_func_e1 = WasmFunction::<(Bid,), (Option<Bid>,)>::new_empty_with_name(&linker, &engine, "pkg:component/nexmark", "all-in-wasm-not-pruned");
                    let r = timed(move |_ctx| {
                        for components in components_bids.unwrap().take(i) {
                            let input = black_box(components);
                            let _output = black_box(wasm_func_e1.switch_default(&input.file));
                        }
                    });
                    wasm_opt3.add(r);
                }
                wasm_opt3.print();
                wasm_opt3.in_file(experiment.as_str(), i);

                println!("Running dynamic wasm opt4 filter...");
                let mut wasm_opt4 = ExperimentResult::new("wasm opt4 dynamic reload", warmup, &output_dir);
                for _ in 0..total {
                    let components_bids = std::fs::File::open(&format!("{dir}/component_bids.csv")).map(iter::<WasmComponent>);
                    let mut wasm_func_e1 = WasmFunction::<(Bid,), (Option<PrunedBid>,)>::new_empty_with_name(&linker, &engine, "pkg:component/nexmark", "all-in-wasm");
                    let r = timed(move |_ctx| {
                        for components in components_bids.unwrap().take(i) {
                            let input = black_box(components);
                            let _output = black_box(wasm_func_e1.switch_default(&input.file));
                        }
                    });
                    wasm_opt4.add(r);
                }
                wasm_opt4.print();
                wasm_opt4.in_file(experiment.as_str(), i);
            }
        },

        _ => panic!("unknown experiment"),
    }
}

fn timed(f: impl FnOnce(&mut Context) + Send + 'static) -> u128 {
    let start = std::time::Instant::now();
    CurrentThreadRunner::run(f);
    let stop = start.elapsed().as_millis();
    // eprintln!("{}", stop);
    stop
}

// Buffered CSV reader
fn iter_with<T: Data + DeserializeOwned + 'static>(file: File, size: usize) -> impl Iterator<Item = T> {
        iter(file).take(size)
}

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
    output_dir: String,
    amount_avg: Option<u128>,
    no_warmup_avg: Option<u128>,
}

impl ExperimentResult {
    pub fn new(name: &str, warmup: u128, output_dir: &String) -> Self {
        ExperimentResult { 
            name: name.to_string(), 
            warmup, durations: vec![], 
            output_dir: output_dir.to_string(), 
            amount_avg: None, 
            no_warmup_avg: None}
    }

    pub fn add(&mut self, r: u128) {
        self.durations.push(r);
    }

    pub fn print(&mut self) {
        let total_count = self.durations.len() as u128;
        let avg_need = self.durations.len() as u128 - self.warmup;
        println!("Experiment: {}", self.name);
        if total_count <= 10 {
            println!("  All: {:?}", self.durations);
        }
        println!("  Processed {} events in {} milliseconds", total_count, self.durations.iter().sum::<u128>());
        self.amount_avg = Some(self.durations.iter().sum::<u128>() / total_count);
        println!("  All {} executions took an average of {} milliseconds", total_count, self.amount_avg.unwrap());
        self.no_warmup_avg = Some(self.durations.iter().rev().take(avg_need as usize).sum::<u128>() / avg_need);
        println!("  The last {} executions took an average of {} milliseconds", avg_need, self.no_warmup_avg.unwrap());
        println!();
    }

    fn timestamp() -> String {
        Utc::now().to_rfc3339()
    }

    pub fn in_file(&self, experiment: &str, size: usize) {
        let file_path = self.output_dir.clone() + "./result/experiment_results_usedonly.csv";
        let is_empty = match File::open(&file_path) {
            Ok(f) => f.metadata().map(|m| m.len() == 0).unwrap_or(true),
            Err(_) => true,
        };
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .expect("Failed to open output file");

        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        if is_empty {
            csv_writer
                .write_record(&["timestamp", "experiment", "name", "size", "amount", "warmup", "duration","amount_avg","no_warmup_avg"])
                .expect("Failed to write header");
        }

        let record = vec![
            Self::timestamp().to_string(),
            experiment.to_string(),
            self.name.clone(),
            size.to_string(),
            self.durations.len().to_string(),
            self.warmup.to_string(),
            format!("{:?}", self.durations),
            self.amount_avg.unwrap().to_string(),
            self.no_warmup_avg.unwrap().to_string(),
        ];

        csv_writer
            .write_record(&record)
            .expect("Failed to write data");
    }
}