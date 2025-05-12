use runtime::prelude::{stream::Event, *};
use crate::{data::{Bid, PrunedBid}, either::Either, wasm::{WasmComponent, WasmFunction}};

#[data]
struct Output {
    auction: u64,
    price: u64,
}

pub fn run_wasm_e2(
    data_stream: Stream<Bid>, 
    components: Stream<WasmComponent>, 
    ctx: &mut Context,
    empty_wasm_func: WasmFunction<(u64,), (bool,)> 
) {
    let data_source = data_stream.map(ctx, |data| Either::Data(data));
    let components_source = components.map(ctx, |component| Either::Component(component));

    let mut input = data_source.merge(ctx, components_source).sorted(ctx);

    ctx.operator(move |tx| async move {
        let mut func = empty_wasm_func;
        loop {
            match input.recv().await {
                Event::Data(time, ref either) => {
                    match either {
                        Either::Component(wasm_component) => {
                            // func.switch(&wasm_component.file, &wasm_component.pkg_name, &wasm_component.name);
                            func.switch_default(&wasm_component.file);
                        },
                        Either::Data(data) => {
                            match func.is_empty() {
                                false => {
                                    let (result,) = func.call((data.auction,));
                                    match result {
                                        true => tx.send(Event::Data(time,Some(Output::new(data.auction, data.price)))).await?,
                                        false => tx.send(Event::Data(time, None)).await?,
                                    }
                                },
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

pub fn run_wasm_operator_g<T>(
    data_stream: Stream<T>, 
    components: Stream<WasmComponent>, 
    ctx: &mut Context,
    empty_wasm_func: WasmFunction<(T,), (Option<T>,)> 
) where 
T: Clone + Unpin + for<'a> runtime::prelude::serde::Deserialize<'a> + runtime::prelude::serde::Serialize + std::fmt::Debug + std::marker::Send+ std::marker::Sync + wasmtime::component::Lower + wasmtime::component::ComponentType + wasmtime::component::Lift + 'static
{
    let data_source = data_stream.map(ctx, |data| Either::Data(data));
    let components_source = components.map(ctx, |component| Either::Component(component));

    let mut input = data_source.merge(ctx, components_source).sorted(ctx);

    ctx.operator(move |tx| async move {
        let mut func = empty_wasm_func;
        loop {
            match input.recv().await {
                Event::Data(time, ref either) => {
                    match either {
                        Either::Component(wasm_component) => {
                            // func.switch(&wasm_component.file, &wasm_component.pkg_name, &wasm_component.name);
                            func.switch_default(&wasm_component.file);
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

pub fn run_wasm_operator(
    data_stream: Stream<Bid>, 
    components: Stream<WasmComponent>, 
    ctx: &mut Context,
    empty_wasm_func: WasmFunction<(Bid,), (Option<PrunedBid>,)> 
) {
    let data_source = data_stream.map(ctx, |data| Either::Data(data));
    let components_source = components.map(ctx, |component| Either::Component(component));

    let mut input = data_source.merge(ctx, components_source).sorted(ctx);

    ctx.operator(move |tx| async move {
        let mut func = empty_wasm_func;
        loop {
            match input.recv().await {
                Event::Data(time, ref either) => {
                    match either {
                        Either::Component(wasm_component) => {
                            // func.switch(&wasm_component.file, &wasm_component.pkg_name, &wasm_component.name);
                            func.switch_default(&wasm_component.file);
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