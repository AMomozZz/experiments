use runtime::prelude::*;
use stream::Event;

use crate::{data::{Bid, QwOutput, QwPrunedBid, WasmComponent}, WasmFunction};

// const GUEST_RS_WASI_MODULE: &[u8] = include_bytes!(concat!(
//     env!("CARGO_MANIFEST_DIR"),
//     "/../../component.wasm"
// ));

#[data]
pub struct Partial {
    pub sum: u64,
    pub count: u64,
    pub max: u64,
    pub min: u64,
    pub sum_sq: u64,
}

impl Partial {
    pub fn identity() -> Self {
        Self::new(0, 0, u64::MIN, u64::MAX, 0)
    }

    pub fn lift(bid: &Bid) -> Self {
        Self::new(bid.price, 1, bid.price, bid.price, bid.price.pow(2))
    }

    pub fn combine(&self, other: &Self) -> Self {
        Self::new(
            self.sum + other.sum,
            self.count + other.count,
            self.max.max(other.max),
            self.min.min(other.min),
            self.sum_sq + other.sum_sq,
        )
    }

    pub fn lower(&self) -> QwOutput {
        let mean = self.sum as f64 / self.count as f64;
        let variance = (self.sum_sq as f64 / self.count as f64) - (mean * mean);
        let stddev = variance.sqrt();
        QwOutput::new(mean, stddev, self.max, self.min)
    }
}

pub fn run(bids: Stream<Bid>, size: usize, step: usize, ctx: &mut Context) {
    let bids = bids.map(ctx, |a| {
        QwPrunedBid::new(a.price)
    });
    bids.count_sliding_holistic_window(ctx, size, step, |data| {
        // let data = data.iter().cloned().collect::<Vec<Bid>>();
        let mut sum = 0;
        let mut count = 0;
        let mut min = u64::MAX;
        let mut max = u64::MIN;
        for bid in data.iter() {
            sum += bid.price;
            count += 1;
            min = min.min(bid.price);
            max = max.max(bid.price);
        }
        let mean = sum as f64 / count as f64;

        let mut sum_sq_diff = 0.0;
        for bid in data.iter() {
            let diff = bid.price as f64 - mean;
            sum_sq_diff += diff * diff;
        }
        let variance = sum_sq_diff / count as f64;
        let stddev = variance.sqrt();
        QwOutput::new(mean, stddev, max, min)
    })
    .drain(ctx);
}

pub fn run_opt(bids: Stream<Bid>, size: usize, step: usize, ctx: &mut Context) {
    bids.count_sliding_aligned_commutative_associative_window(
        ctx,
        size,
        step,
        Partial::identity(),
        Partial::lift,
        Partial::combine,
        Partial::lower,
    )
    .drain(ctx);
}

pub fn run_wasm(bids: Stream<Bid>, size: usize, step: usize, ctx: &mut Context, wasm_func: WasmFunction<(Vec<QwPrunedBid>,), (QwOutput,)>) {
    let bids = bids.map(ctx, |a| {
        QwPrunedBid::new(a.price)
    });
    // wasm_func.switch(GUEST_RS_WASI_MODULE, "pkg:component/nexmark", "qw");
    bids.count_sliding_holistic_window(ctx, size, step, move |data| {
        // wasm_func.call((data.iter().cloned().collect::<Vec<Bid>>(),)).0
        wasm_func.call((data.to_vec(),)).0
    })
    .drain(ctx);
}

pub fn run_wasm_operator(
    mut data: Stream<Bid>, 
    mut components: Stream<WasmComponent>, 
    ctx: &mut Context,
    empty_wasm_func: WasmFunction<(Bid,), (QwOutput,)> 
) {
    ctx.operator(move |tx: stream::Collector<QwOutput>| async move {
        // initialise WASM
        let mut func = Some(empty_wasm_func);
        tokio::select! {
            event = components.recv() => {
                loop {
                    match event {
                        Event::Data(_time, ref wasm_component) => {
                            // update func
                            if let Some(ref mut f) = func {
                                f.switch_default(&wasm_component.file);
                                // tx.send(Event::Data(_time, O::new_empty())).await?;
                            }
                        },
                        Event::Watermark(time) => tx.send(Event::Watermark(time)).await?,
                        Event::Snapshot(id) => tx.send(Event::Snapshot(id)).await?,
                        Event::Sentinel => {
                            tx.send(Event::Sentinel).await?;
                            break;
                        },
                    }
                }
            },
            event = data.recv() => {
                loop {
                    match event {
                        Event::Data(time, ref data) => {
                            if let Some(ref mut f) = func {
                                // Call the function with the data
                                // f.call(data);
                                tx.send(Event::Data(time, f.call((data.clone(),)).0)).await?
                            }
                        },
                        Event::Watermark(time) => tx.send(Event::Watermark(time)).await?,
                        Event::Snapshot(id) => tx.send(Event::Snapshot(id)).await?,
                        Event::Sentinel => {
                            tx.send(Event::Sentinel).await?;
                            break;
                        },
                    }
                }
                
            },
        }
        Ok(())
    })
    .drain(ctx);
}