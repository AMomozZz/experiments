@echo off
setlocal enabledelayedexpansion

@REM echo Building the project...
@REM cargo component build --target wasm32-wasip2 --release
@REM @REM rustc ./src/main.rs --target wasm32-wasip2
@REM if %errorlevel% neq 0 (
@REM         echo Build failed.
@REM         exit /b %errorlevel%
@REM )

echo Running with Wasmtime...
@REM wasmtime -S inherit-network=y .\target\wasm32-wasip1\release\host.wasm 127 0 0 1 8080 127 0 0 1 8090 127 0 0 1 8100
@REM wasmtime -S inherit-network=y ./main.wasm 127 0 0 1 8080 127 0 0 1 8090 127 0 0 1 8100

@REM e1 & e3 * 3
@REM ./../guest/target/wasm32-wasip2/release/component.wasm
@REM ./../guest/target/wasm32-wasip2/release/component_usedonly.wasm
@REM ./../guest/target/wasm32-wasip2/release/component_usedonly_opt.wasm
@REM cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bid e1 100 25 ./host/result/e1_e3_e5/experiment_results default
@REM cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bid e3 100 25 ./host/result/e1_e3_e5/experiment_results default
@REM cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bid e1 100 25 ./host/result/e1_e3_e5/experiment_results_usedonly usedonly
@REM cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bid e3 100 25 ./host/result/e1_e3_e5/experiment_results_usedonly usedonly
@REM cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bid e1 100 25 ./host/result/e1_e3_e5/experiment_results_usedonly_opt usedonly_opt
@REM cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bid e3 100 25 ./host/result/e1_e3_e5/experiment_results_usedonly_opt usedonly_opt


@REM e2 & e4
cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bidComponent100 e2 100 25 ./host/result/e2_e4_e5/bidComponent100
cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bidComponent100 e4 100 25 ./host/result/e2_e4_e5/bidComponent100
cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bidComponent10000 e2 100 25 ./host/result/e2_e4_e5/bidComponent10000
cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bidComponent10000 e4 100 25 ./host/result/e2_e4_e5/bidComponent10000
cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bidComponent100_usedonly e2 100 25 ./host/result/e2_e4_e5/bidComponent100_usedonly
cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bidComponent100_usedonly e4 100 25 ./host/result/e2_e4_e5/bidComponent100_usedonly
cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bidComponent10000_usedonly e2 100 25 ./host/result/e2_e4_e5/bidComponent10000_usedonly
cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bidComponent10000_usedonly e4 100 25 ./host/result/e2_e4_e5/bidComponent10000_usedonly
cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bidComponent100_usedonly_opt e2 100 25 ./host/result/e2_e4_e5/bidComponent100_usedonly_opt
cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bidComponent100_usedonly_opt e4 100 25 ./host/result/e2_e4_e5/bidComponent100_usedonly_opt
cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bidComponent10000_usedonly_opt e2 100 25 ./host/result/e2_e4_e5/bidComponent10000_usedonly_opt
cargo r --release --manifest-path=host/Cargo.toml -- nexmark-data/bidComponent10000_usedonly_opt e4 100 25 ./host/result/e2_e4_e5/bidComponent10000_usedonly_opt