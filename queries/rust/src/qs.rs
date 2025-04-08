use runtime::prelude::*;
use stream::Event;

use crate::{data::{Bid, WasmComponent}, WasmFunction};

// const GUEST_RS_WASI_MODULE: &[u8] = include_bytes!(concat!(
//     env!("CARGO_MANIFEST_DIR"),
//     "/../../component.wasm"
// ));

pub fn run_wasm_operator(
    mut data: Stream<Bid>, 
    mut components: Stream<WasmComponent>, 
    ctx: &mut Context,
    empty_wasm_func: WasmFunction<(Bid,), (Option<Bid>,)> 
) {
    ctx.operator(move |tx| async move {
        // initialise WASM
        let mut func = empty_wasm_func;
        // match components.recv().await {
        //     Event::Data(_time, ref wasm_component) => {
        //         // update func
        //         if let Some(ref mut f) = func {
        //             eprintln!("{:?}", wasm_component);
        //             f.switch(&wasm_component.file, &wasm_component.pkg_name, &wasm_component.name);
        //             // tx.send(Event::Data(_time, O::new_empty())).await?;
        //         }
        //     },
        //     Event::Watermark(time) => tx.send(Event::Watermark(time)).await?,
        //     Event::Snapshot(id) => tx.send(Event::Snapshot(id)).await?,
        //     Event::Sentinel => {
        //         tx.send(Event::Sentinel).await?;
        //     },
        // };

        loop {
            tokio::select! {
                event = components.recv() => {
                    // loop {
                        match event {
                            Event::Data(_time, ref wasm_component) => {
                                // update func
                                // match func.is_empty() {
                                //     true => func.switch(&wasm_component.file, &wasm_component.pkg_name, &wasm_component.name),
                                //     false => 
                                // } 
                                func.switch(&wasm_component.file, &wasm_component.pkg_name, &wasm_component.name);
                            },
                            Event::Watermark(time) => tx.send(Event::Watermark(time)).await?,
                            Event::Snapshot(id) => tx.send(Event::Snapshot(id)).await?,
                            Event::Sentinel => {
                                tx.send(Event::Sentinel).await?;
                                break;
                            },
                        }
                    // }
                },
                event = data.recv() => {
                    // loop {
                        match event {
                            Event::Data(time, ref data) => {
                                match func.is_empty() {
                                    false => tx.send(Event::Data(time, func.call((data.clone(),)).0)).await?,
                                    true => {
                                        eprintln!("{}", data.date_time);
                                        tx.send(Event::Data(time, None)).await?
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
                    // }
                    
                },
            }
        }
        Ok(())
    })
    .drain(ctx);
}