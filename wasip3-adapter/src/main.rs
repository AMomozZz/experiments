use std::env;
use std::path::PathBuf;
use std::process::Command;

use wasmparser::Validator;
use wasmparser::WasmFeatures;
use wit_component::ComponentEncoder;

fn main() {
    let out_dir = PathBuf::from(r#"D:\master\thesis\aqualang\webassembly\experiments\wasip3-adapter\target"#);

    let guest = env::current_dir().unwrap().parent().unwrap().join("guest-rs");

    let status = Command::new("cargo")
        .current_dir(&guest)
        .arg("build")
        .arg("--target")
        .arg("wasm32-wasip1")
        .env("CARGO_TARGET_DIR", &out_dir)
        .status()
        .unwrap();

    if !status.success() {
        panic!("Failed to build {}", guest.display());
    }

    let wasip3_prototyping = env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("wasip3-prototyping");

    let status = Command::new("cargo")
        .current_dir(&wasip3_prototyping)
        .arg("build")
        .arg("--package")
        .arg("wasi-preview1-component-adapter")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        // Ensure we use the same version of `wasm-encoder`
        .arg("--config")
        .arg(r#"patch.crates-io.wasm-encoder.version="0.228.0""#)
        .arg("--config")
        .arg(r#"patch.crates-io.wasm-encoder.git="https://github.com/bytecodealliance/wasm-tools""#)
        .arg("--config")
        .arg(r#"patch.crates-io.wasm-encoder.rev="df50cdf49b41656351d603a8f99bae11c50ea524""#)
        .env("CARGO_TARGET_DIR", &out_dir)
        .status()
        .unwrap();

    if !status.success() {
        panic!("Failed to build {}", wasip3_prototyping.display());
    }

    let module = std::fs::read(
        out_dir
            .join("wasm32-wasip1")
            .join("debug")
            .join("component_stream.wasm"),
    )
    .unwrap();

    let adapter = std::fs::read(
        out_dir
            .join("wasm32-unknown-unknown")
            .join("debug")
            .join("wasi_snapshot_preview1.wasm"),
    )
    .unwrap();

    let component = ComponentEncoder::default()
        .module(module.as_slice())
        .unwrap()
        .validate(false)
        .adapter("wasi_snapshot_preview1", &adapter)
        .unwrap()
        .encode()
        .expect("module can be translated to a component");

    Validator::new_with_features(WasmFeatures::all())
        .validate_all(&component)
        .expect("component output should validate");

    std::fs::write(
        &out_dir
            .join("wasm32-wasip1")
            .join("debug")
            .join("guest.component.wasm"),
        component,
    )
    .expect("write component to disk");

    println!("cargo:rerun-if-changed={}", out_dir.display());
}
