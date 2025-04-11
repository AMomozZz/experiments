@echo off
setlocal enabledelayedexpansion

@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid q1-wasm
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid q2-wasm
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid q2-wasm-sf
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid q2-wasm-mf
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid q2-wasm-mf-opt
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/auctionPerson q3-wasm
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/auctionBid q4-wasm-s
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/auctionBid q4-wasm-m
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid q5-wasm
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/auctionBid q6-wasm
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/auctionBid q6-wasm-ng
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid q7-wasm
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid qw-wasm 100 1


@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid qw 100 100
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid qw 1000 100
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid qw 10000 100
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid qw 10000 10000

@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid qw-opt 100 100
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid qw-opt 1000 100
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid qw-opt 10000 100
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid qw-opt 10000 10000

@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid qw-wasm 100 100
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid qw-wasm 1000 100
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid qw-wasm 10000 100
@REM cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid qw-wasm 10000 10000

cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid io
cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bidComponent io
cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bidComponentG io
cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bidComponentG io-with-map
cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bidComponentG io-datas
cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bidComponentG io-components
cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bidComponentG switch-component

cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid q1-opt
cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bid q2-opt
cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bidComponent qs-wasm
cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/bidComponentG qs-wasm-g