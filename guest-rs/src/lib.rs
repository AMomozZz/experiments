wit_bindgen::generate!({
    world: "guest",
});

use exports::pkg::component::nexmark::{Bid, Guest};
use wit_bindgen::StreamReader;
use wit_bindgen::rt::async_support;
struct Component;

export!(Component);

impl Guest for Component {
    fn q1(mut stream: StreamReader<Bid>,) -> StreamReader<Bid> {
            let (mut tx, rx) = wit_stream::new::<Bid>();
            async_support::spawn(async move {
                loop {
                    match stream.next().await {
                        Some(ref item) => {
                            let mut re = item.clone();
                            re.price = item.price * 100 / 85;
                            tx.write(vec![re]).await;
                        }
                        _ => {
                            break;
                        }
                    }
                }
            });
            rx
        }
}
