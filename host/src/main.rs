pub mod data;
pub mod wasm;
pub mod either;
pub mod e1;
pub mod e2;
pub mod e3;
pub mod experiment_framework;

use std::fs::File;
use runtime::prelude::Context;
use experiment_framework::{iter_with, iter, stream, stream_with, timed, ExperimentResult, WATERMARK_FREQUENCY};
use data::{Bid, PrunedBid};
use wasm::{Host, WasmComponent, WasmFunction};
use wasmtime::{component::Linker, Config, Engine};
use wasmtime_wasi::WasiImpl;
use std::hint::black_box;

const USAGE: &str = "Usage: cargo run <data-dir> <experiment> <variant> <measure-experiment-num> <warmup-num> <output-path> [wasm-type]";

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
    let Some(variant) = args.next() else {
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
    
    // 获取 WASM 类型参数（可选）
    let wasm_type = args.next().unwrap_or_else(|| "default".to_string());
    
    let mut guest_rs_wasi_module: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "./../guest/target/wasm32-wasip2/release/component.wasm"
    ));
    
    // 根据 wasm_type 选择不同的 WASM 模块
    match wasm_type.as_str() {
        "usedonly" => {
            guest_rs_wasi_module = include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "./../guest/target/wasm32-wasip2/release/component_usedonly.wasm"
            ));
        }
        "usedonly_opt" => {
            guest_rs_wasi_module = include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "./../guest/target/wasm32-wasip2/release/component_usedonly_opt.wasm"
            ));
        }
        _ => {} // 使用默认的
    };

    let total: u128 = measure + warmup;
    let warmup: u128 = warmup;

    let config = Config::new();
    let engine = Engine::new(&config).unwrap();
    let mut linker = Linker::new(&engine);
    
    wasmtime_wasi::add_to_linker_sync::<WasiImpl<Host>>(&mut linker).unwrap();

    // 根据 experiment 和 variant 分发
    match experiment.as_str() {
        "e1" => run_e1_variant(&variant, &dir, total, warmup, &output_dir, &linker, &engine, guest_rs_wasi_module),
        "e2" => run_e2_variant(&variant, &dir, total, warmup, &output_dir, &linker, &engine, guest_rs_wasi_module),
        "e3" => run_e3_variant(&variant, &dir, total, warmup, &output_dir, &linker, &engine, guest_rs_wasi_module),
        "e4" => run_e4_variant(&variant, &dir, total, warmup, &output_dir, &linker, &engine, guest_rs_wasi_module),
        _ => panic!("unknown experiment: {}", experiment),
    }
}

fn run_e1_variant(
    variant: &str,
    dir: &str,
    total: u128,
    warmup: u128,
    output_dir: &str,
    linker: &Linker<WasiImpl<Host>>,
    engine: &Engine,
    guest_module: &[u8],
) {
    let sizes = vec![100, 1000, 10000, 100000, 1000000];
    
    for &size in &sizes {
        println!("Running e1::{} with size {}", variant, size);
        
        match variant {
            "io" => {
                let mut result = ExperimentResult::new("io", warmup, &output_dir.to_string());
                for _ in 0..total {
                    let bids = File::open(&format!("{dir}/bids.csv"))
                        .map(|file| iter_with::<Bid>(file, size));
                    let r = timed(move |ctx| stream(ctx, bids).drain(ctx));
                    result.add(r);
                }
                result.print();
                result.in_file("e1", size);
            }
            
            "native_opt" => {
                let mut result = ExperimentResult::new("native_opt", warmup, &output_dir.to_string());
                for _ in 0..total {
                    let bids = File::open(&format!("{dir}/bids.csv"))
                        .map(|file| iter_with::<Bid>(file, size));
                    let r = timed(move |ctx| e1::run_opt(stream(ctx, bids), ctx));
                    result.add(r);
                }
                result.print();
                result.in_file("e1", size);
            }
            
            "wasm_pass_all" => {
                let mut result = ExperimentResult::new("wasm_pass_all", warmup, &output_dir.to_string());
                for _ in 0..total {
                    let bids = File::open(&format!("{dir}/bids.csv"))
                        .map(|file| iter_with::<Bid>(file, size));
                    let wasm_func_q2 = WasmFunction::<(u64, u64, Vec<u64>,), (Option<(u64, u64)>,)>::new(
                        linker, engine, guest_module, "pkg:component/nexmark", "q2"
                    );
                    let r = timed(move |ctx| e1::run_wasm(stream(ctx, bids), ctx, wasm_func_q2));
                    result.add(r);
                }
                result.print();
                result.in_file("e1", size);
            }
            
            "wasm_opt_pruned" => {
                let mut result = ExperimentResult::new("wasm_opt_pruned", warmup, &output_dir.to_string());
                for _ in 0..total {
                    let bids = File::open(&format!("{dir}/bids.csv"))
                        .map(|file| iter_with::<Bid>(file, size));
                    let wasm_func_single_filter = WasmFunction::<(u64, Vec<u64>,), (bool,)>::new(
                        linker, engine, guest_module, "pkg:component/nexmark", "single-filter"
                    );
                    let r = timed(move |ctx| e1::run_wasm_sf(stream(ctx, bids), ctx, wasm_func_single_filter));
                    result.add(r);
                }
                result.print();
                result.in_file("e1", size);
            }
            
            "wasm_opt2" => {
                let mut result = ExperimentResult::new("wasm_opt2", warmup, &output_dir.to_string());
                for _ in 0..total {
                    let bids = File::open(&format!("{dir}/bids.csv"))
                        .map(|file| iter_with::<Bid>(file, size));
                    let wasm_func_e1 = WasmFunction::<(u64,), (bool,)>::new(
                        linker, engine, guest_module, "pkg:component/nexmark", "e1"
                    );
                    let r = timed(move |ctx| e1::run_wasm_e1(stream(ctx, bids), ctx, wasm_func_e1));
                    result.add(r);
                }
                result.print();
                result.in_file("e1", size);
            }
            
            "wasm_opt3" => {
                let mut result = ExperimentResult::new("wasm_opt3", warmup, &output_dir.to_string());
                for _ in 0..total {
                    let bids = File::open(&format!("{dir}/bids.csv"))
                        .map(|file| iter_with::<Bid>(file, size));
                    let wasm_func_e1 = WasmFunction::<(Bid,), (Option<Bid>,)>::new(
                        linker, engine, guest_module, "pkg:component/nexmark", "all-in-wasm-not-pruned"
                    );
                    let r = timed(move |ctx| e1::run_wasm_e1_all_in_wasm_g::<Bid>(stream(ctx, bids), ctx, wasm_func_e1));
                    result.add(r);
                }
                result.print();
                result.in_file("e1", size);
            }
            
            "wasm_opt4" => {
                let mut result = ExperimentResult::new("wasm_opt4", warmup, &output_dir.to_string());
                for _ in 0..total {
                    let bids = File::open(&format!("{dir}/bids.csv"))
                        .map(|file| iter_with::<Bid>(file, size));
                    let wasm_func_e1 = WasmFunction::<(Bid,), (Option<PrunedBid>,)>::new(
                        linker, engine, guest_module, "pkg:component/nexmark", "all-in-wasm"
                    );
                    let r = timed(move |ctx| e1::run_wasm_e1_all_in_wasm(stream(ctx, bids), ctx, wasm_func_e1));
                    result.add(r);
                }
                result.print();
                result.in_file("e1", size);
            }
            
            _ => panic!("unknown e1 variant: {}", variant),
        }
    }
}

fn run_e2_variant(
    variant: &str,
    dir: &str,
    total: u128,
    warmup: u128,
    output_dir: &str,
    linker: &Linker<WasiImpl<Host>>,
    engine: &Engine,
    guest_module: &[u8],
) {
    // E2 只需要运行一次，不需要 size 循环
    println!("Running e2::{}", variant);
    
    match variant {
        "io" => {
            let component_len = File::open(&format!("{dir}/component_bids.csv"))
                .map(iter::<WasmComponent>).unwrap().count();
            let mut result = ExperimentResult::new("io", warmup, &output_dir.to_string());
            for _ in 0..total {
                let bids = File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                let components_bids = File::open(&format!("{dir}/component_bids.csv")).map(iter::<WasmComponent>);
                let r = timed(move |ctx| {
                    stream(ctx, bids).drain(ctx);
                    stream_with(ctx, components_bids, 1).drain(ctx);
                });
                result.add(r);
            }
            result.print();
            result.in_file("e2", component_len);
        }
        
        "wasm_opt2" => {
            let component_len = File::open(&format!("{dir}/component_bids.csv"))
                .map(iter::<WasmComponent>).unwrap().count();
            let mut result = ExperimentResult::new("wasm_opt2_dynamic", warmup, &output_dir.to_string());
            for _ in 0..total {
                let bids = File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                let components_bids = File::open(&format!("{dir}/component_bids.csv")).map(iter::<WasmComponent>);
                let wasm_func_e1 = WasmFunction::<(u64,), (bool,)>::new_empty_with_name(
                    linker, engine, "pkg:component/nexmark", "e1"
                );
                let r = timed(move |ctx| {
                    e2::run_wasm_e2(stream(ctx, bids), stream_with(ctx, components_bids, 1), ctx, wasm_func_e1)
                });
                result.add(r);
            }
            result.print();
            result.in_file("e2", component_len);
        }
        
        // ... 其他 e2 variants
        _ => panic!("unknown e2 variant: {}", variant),
    }
}

fn run_e3_variant(
    variant: &str,
    dir: &str,
    total: u128,
    warmup: u128,
    output_dir: &str,
    linker: &Linker<WasiImpl<Host>>,
    engine: &Engine,
    guest_module: &[u8],
) {
    let sizes = vec![100, 1000, 10000, 100000, 1000000];
    
    for &size in &sizes {
        println!("Running e3::{} with size {}", variant, size);
        
        match variant {
            "io" => {
                let mut result = ExperimentResult::new("io", warmup, &output_dir.to_string());
                for _ in 0..total {
                    let bids = File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                    let r = timed(move |_ctx| {
                        for bid in bids.unwrap().take(size) {
                            let _input = black_box(bid);
                        }
                    });
                    result.add(r);
                }
                result.print();
                result.in_file("e3", size);
            }
            
            "native_opt" => {
                let mut result = ExperimentResult::new("native_opt", warmup, &output_dir.to_string());
                for _ in 0..total {
                    let bids = File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                    let r = timed(move |_ctx| {
                        for bid in bids.unwrap().take(size) {
                            let input = black_box(bid);
                            let _output = black_box(e3::opt_func(input));
                        }
                    });
                    result.add(r);
                }
                result.print();
                result.in_file("e3", size);
            }
            
            "wasm_pass_all" => {
                let mut result = ExperimentResult::new("wasm_pass_all", warmup, &output_dir.to_string());
                for _ in 0..total {
                    let bids = File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
                    let wasm_func_q2 = WasmFunction::<(u64, u64, Vec<u64>,), (Option<(u64, u64)>,)>::new(
                        linker, engine, guest_module, "pkg:component/nexmark", "q2"
                    );
                    let r = timed(move |_ctx| {
                        for bid in bids.unwrap().take(size) {
                            let input = black_box(bid);
                            let _output = black_box(e3::run_wasm_func(input, |args| wasm_func_q2.call(args)));
                        }
                    });
                    result.add(r);
                }
                result.print();
                result.in_file("e3", size);
            }
            
            // ... 其他 e3 variants
            _ => panic!("unknown e3 variant: {}", variant),
        }
    }
}

fn run_e4_variant(
    variant: &str,
    dir: &str,
    total: u128,
    warmup: u128,
    output_dir: &str,
    linker: &Linker<WasiImpl<Host>>,
    engine: &Engine,
    guest_module: &[u8],
) {
    let sizes = vec![1, 10, 100, 1000, 10000];
    
    for &size in &sizes {
        println!("Running e4::{} with size {}", variant, size);
        
        match variant {
            "io" => {
                let mut result = ExperimentResult::new("io", warmup, &output_dir.to_string());
                for _ in 0..total {
                    let components_bids = File::open(&format!("{dir}/component_bids.csv")).map(iter::<WasmComponent>);
                    let r = timed(move |_ctx| {
                        for components in components_bids.unwrap().take(size) {
                            let _input = black_box(components);
                        }
                    });
                    result.add(r);
                }
                result.print();
                result.in_file("e4", size);
            }
            
            // ... 其他 e4 variants
            _ => panic!("unknown e4 variant: {}", variant),
        }
    }
}