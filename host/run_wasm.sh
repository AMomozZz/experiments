#!/bin/bash

set -e  # 如果命令出错则退出

echo "Building the project..."
cargo build --target wasm32-wasip2 --release

echo "Running with Wasmtime..."
wasmtime -S inherit-network=y ./target/wasm32-wasip2/release/host.wasm 127 0 0 1 8080
