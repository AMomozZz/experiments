use runtime::prelude::*;
use stream::Event;

use crate::{data::Bid, either::{Either, EitherData}, wasm::{WasmComponent, WasmFunction}};

pub fn run_wasm_operator(
    data: Stream<Bid>, 
    components: Stream<WasmComponent>, 
    ctx: &mut Context,
    empty_wasm_func: WasmFunction<(Bid,), (Option<Bid>,)> 
) {
    let data_source = data.map(ctx, |bid| Either::Data(EitherData::Bid(bid)));
    let components_source = components.map(ctx, |component| Either::Component(component));

    let mut input = data_source.merge(ctx, components_source).sorted(ctx);

    ctx.operator(move |tx| async move {
        let mut func = empty_wasm_func;
        loop {
            match input.recv().await {
                Event::Data(time, ref either) => {
                    match either {
                        Either::Component(wasm_component) => {
                            func.switch(&wasm_component.file, &wasm_component.pkg_name, &wasm_component.name);
                        },
                        Either::Data(data) => {
                            match data {
                                EitherData::Auction(_auction) => todo!(),
                                EitherData::Bid(bid) => match func.is_empty() {
                                    false => tx.send(Event::Data(time, func.call((bid.clone(),)).0)).await?,
                                    true => tx.send(Event::Data(time, None)).await?,
                                },
                                EitherData::Person(_person) => todo!(),
                            }
                        },
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
        Ok(())
    })
    .drain(ctx);
}

pub fn run_wasm_operator_opt(
    data: Stream<Bid>, 
    components: Stream<WasmComponent>, 
    ctx: &mut Context,
    empty_wasm_func: WasmFunction<(Bid,), (Option<Bid>,)> 
) {
    let data_source = data.map(ctx, |bid| Either::Data(EitherData::Bid(bid)));
    let components_source = components.map(ctx, |component| Either::Component(component));

    // let mut input = data_source.sorted_merge(ctx, components_source);
    let mut input = data_source.merge(ctx, components_source).sorted_heap(ctx);

    ctx.operator(move |tx| async move {
        let mut func = empty_wasm_func;
        loop {
            match input.recv().await {
                Event::Data(time, ref either) => {
                    match either {
                        Either::Component(wasm_component) => {
                            func.switch(&wasm_component.file, &wasm_component.pkg_name, &wasm_component.name);
                        },
                        Either::Data(data) => {
                            match data {
                                EitherData::Auction(_auction) => todo!(),
                                EitherData::Bid(bid) => match func.is_empty() {
                                    false => tx.send(Event::Data(time, func.call((bid.clone(),)).0)).await?,
                                    true => tx.send(Event::Data(time, None)).await?,
                                },
                                EitherData::Person(_person) => todo!(),
                            }
                        },
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
        Ok(())
    })
    .drain(ctx);
}

pub fn run_wasm_operator_g(
    datas: Stream<EitherData>, 
    components: Stream<WasmComponent>, 
    ctx: &mut Context,
    empty_wasm_func: WasmFunction<(EitherData,), (Option<EitherData>,)>
) {
    let datas_source = datas.map(ctx, |data| Either::Data(data));
    let components_source = components.map(ctx, |component| Either::Component(component));

    let mut input = datas_source.merge(ctx, components_source).sorted(ctx);

    ctx.operator(move |tx| async move {
        let mut func = empty_wasm_func;
        loop {
            match input.recv().await {
                Event::Data(time, ref either) => {
                    match either {
                        Either::Component(wasm_component) => {
                            func.switch(&wasm_component.file, &wasm_component.pkg_name, &wasm_component.name);
                        },
                        Either::Data(data) => {
                            match func.is_empty() {
                                false => tx.send(Event::Data(time, func.call((data.clone(),)).0)).await?,
                                true => tx.send(Event::Data(time, None)).await?,
                            }
                        },
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
        Ok(())
    })
    .drain(ctx);
}