use runtime::prelude::*;
use stream::Event;

use crate::{data::{Bid, WasmComponent}, either::{Either, EitherData}, WasmFunction};

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
                        // Either::Bid(bid) => {
                        //     match func.is_empty() {
                        //         false => tx.send(Event::Data(time, func.call((bid.clone(),)).0)).await?,
                        //         true => tx.send(Event::Data(time, None)).await?,
                        //     }
                        // },
                        Either::Component(wasm_component) => {
                            func.switch(&wasm_component.file, &wasm_component.pkg_name, &wasm_component.name);
                        },
                        // Either::Auction(_auction) => todo!(),
                        // Either::Person(_person) => todo!(),
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

// pub fn run_wasm_operator_all(
//     option_bids: Option<Stream<Bid>>, 
//     option_auctions: Option<Stream<Auction>>,
//     option_persons: Option<Stream<Person>>,
//     components: Stream<WasmComponent>, 
//     ctx: &mut Context,
//     empty_wasm_func: WasmFunction<(Bid,), (Option<Bid>,)>
// ) {
//     let components_source = components.map(ctx, |component| Either::Component(component));
//     let bids_source = match option_bids {
//         Some(bids) => bids.map(ctx, |bid| Either::Bid(bid)),
//         None => Stream::new().1.map(ctx, |bid| Either::Bid(bid)),
//     };
//     let auction_sourse = match option_auctions {
//         Some(auctions) => auctions.map(ctx, |auction| Either::Auction(auction)),
//         None => Stream::new().1.map(ctx, |auction| Either::Bid(auction)),
//     };
//     let persons_sourse = match option_persons {
//         Some(persons) => persons.map(ctx, |person| Either::Person(person)),
//         None => Stream::new().1.map(ctx, |person| Either::Person(person)),
//     };

//     let mut input = components_source.merge(ctx, bids_source).merge(ctx, auction_sourse).merge(ctx, persons_sourse).sorted(ctx);

//     ctx.operator(move |tx| async move {
//         let mut func = empty_wasm_func;
//         loop {
//             match input.recv().await {
//                 Event::Data(time, ref either) => {
//                     match either {
//                         Either::Bid(data) => {
//                             match func.is_empty() {
//                                 false => tx.send(Event::Data(time, func.call((data.clone(),)).0)).await?,
//                                 true => tx.send(Event::Data(time, None)).await?,
//                             }
//                         },
//                         Either::Component(wasm_component) => {
//                             func.switch(&wasm_component.file, &wasm_component.pkg_name, &wasm_component.name);
//                         },
//                         Either::Auction(auction) => {
//                             match func.is_empty() {
//                                 false => tx.send(Event::Data(time, func.call((auction.clone(),)).0)).await?,
//                                 true => tx.send(Event::Data(time, None)).await?,
//                             }
//                         },
//                         Either::Person(person) => {
//                             match func.is_empty() {
//                                 false => tx.send(Event::Data(time, func.call((person.clone(),)).0)).await?,
//                                 true => tx.send(Event::Data(time, None)).await?,
//                             }
//                         },
//                     }
//                 },
//                 Event::Watermark(time) => tx.send(Event::Watermark(time)).await?,
//                 Event::Snapshot(id) => tx.send(Event::Snapshot(id)).await?,
//                 Event::Sentinel => {
//                     tx.send(Event::Sentinel).await?;
//                     break;
//                 },
//             }
//         }
//         Ok(())
//     })
//     .drain(ctx);
// }