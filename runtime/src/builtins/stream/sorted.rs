use crate::builtins::stream::Event;
use crate::builtins::time::Time;
use crate::runner::context::Context;
use crate::traits::Data;

use super::Stream;

impl<T: Data> Stream<T> {
    pub fn sorted(mut self, ctx: &mut Context) -> Self {
        ctx.operator(|tx| async move {
            let mut buffer: Vec<Event<T>> = Vec::new();

            loop {
                match self.recv().await {
                    Event::Data(t, v) => buffer.push(Event::Data(t, v)),
                    Event::Watermark(w) => {
                        let current_watermark = w;

                        let (mut ready, pending): (Vec<_>, Vec<_>) = buffer
                            .into_iter()
                            .partition(|e| match e {
                                Event::Data(t, _) => *t <= current_watermark,
                                _ => false,
                            });

                        buffer = pending;
                        ready.sort_by_key(|e| match e {
                            Event::Data(t, _) => *t,
                            _ => Time::zero(),
                        });

                        for event in ready {
                            tx.send(event).await?;
                        }

                        tx.send(Event::Watermark(w)).await?;
                    },
                    Event::Snapshot(id) => tx.send(Event::Snapshot(id)).await?,
                    Event::Sentinel => {
                        buffer.sort_by_key(|e| match e {
                            Event::Data(t, _) => *t,
                            _ => Time::zero(),
                        });

                        for event in buffer.drain(..) {
                            tx.send(event).await?;
                        }

                        tx.send(Event::Sentinel).await?;
                        break;
                    },
                }
            }
            Ok(())
        })
    }
}