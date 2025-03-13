# Nexmark Benchmark

This directory contains the Nexmark benchmark implemented for Aqua, targeting Flink and Rust.

The benchmark contains 2 experiments:
* Experiment 1: The 8 standard Nexmark Queries.
* Experiment 2: A custom Sliding Window Aggregation query evaluated for different window sizes using bid data provided by Nexmark.

## Running

To run the experiments, install and startup [docker](https://docs.docker.com/) and then run:

```bash
./docker.sh
```

Or

```bash
cd root/
cargo r --manifest-path=data-generator/Cargo.toml -- --num-events 1000000 --bids --dir nexmark-data/q1
```

```bash
cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/q1 io
```

and

```bash
cargo r --release --manifest-path=queries/rust/Cargo.toml -- nexmark-data/q1 q1
```

## Output

After running, plots of the experiments can be found in the generated `output/` folder.
