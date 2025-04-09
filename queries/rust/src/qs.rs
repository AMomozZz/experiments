use runtime::prelude::*;
use stream::Event;

use crate::{data::{Bid, WasmComponent}, either::Either, WasmFunction};

pub fn merge_and_sort(
    data: Stream<Bid>, 
    components: Stream<WasmComponent>, 
    ctx: &mut Context,
) -> Stream<Either> {
    let data_source = data.map(ctx, |bid| Either::Bid(bid));
    let components_source = components.map(ctx, |component| Either::Component(component));

    data_source.merge(ctx, components_source).sorted(ctx)
}

pub fn run_wasm_operator(
    mut data: Stream<Bid>, 
    mut components: Stream<WasmComponent>, 
    ctx: &mut Context,
    empty_wasm_func: WasmFunction<(Bid,), (Option<Bid>,)> 
) {
    ctx.operator(move |tx| async move {
        let mut func = empty_wasm_func;
        loop {
            tokio::select! {
                event = components.recv() => {
                    match event {
                        Event::Data(_time, ref wasm_component) => {
                            func.switch(&wasm_component.file, &wasm_component.pkg_name, &wasm_component.name);
                        },
                        Event::Watermark(time) => tx.send(Event::Watermark(time)).await?,
                        _ => {},
                    }
                },
                event = data.recv() => {
                    match event {
                        Event::Data(time, ref data) => {
                            match func.is_empty() {
                                false => tx.send(Event::Data(time, func.call((data.clone(),)).0)).await?,
                                true => tx.send(Event::Data(time, None)).await?,
                            }
                        },
                        Event::Watermark(time) => {
                            // eprintln!("water {}", time);
                            tx.send(Event::Watermark(time)).await?
                        },
                        Event::Snapshot(id) => tx.send(Event::Snapshot(id)).await?,
                        Event::Sentinel => {
                            tx.send(Event::Sentinel).await?;
                            break;
                        },
                    }
                },
            }
        }
        Ok(())
    })
    .drain(ctx);
}