use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::builtins::stream::Event;
use crate::builtins::time::Time;
use crate::runner::context::Context;
use crate::traits::Data;

use super::Stream;

struct HeapEntry<T> {
    time: Time,
    seq: usize,
    event: Event<T>,
}

impl<T> PartialEq for HeapEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.seq == other.seq
    }
}

impl<T> Eq for HeapEntry<T> {}

impl<T> PartialOrd for HeapEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.cmp(self)) // Reverse order for min-heap
    }
}

impl<T> Ord for HeapEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time).then(self.seq.cmp(&other.seq))
    }
}

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

    pub fn sorted_heap(mut self, ctx: &mut Context) -> Self {
        ctx.operator(|tx| async move {
            let mut heap: BinaryHeap<HeapEntry<T>> = BinaryHeap::new();
            let mut seq = 0;
    
            loop {
                match self.recv().await {
                    Event::Data(t, v) => {
                        heap.push(HeapEntry {
                            time: t,
                            seq,
                            event: Event::Data(t, v),
                        });
                        seq += 1;
                    },
                    Event::Watermark(w) => {
                        let current_watermark = w;
    
                        while let Some(peek) = heap.peek() {
                            if peek.time <= current_watermark {
                                let HeapEntry { event, .. } = heap.pop().unwrap();
                                tx.send(event).await?;
                            } else {
                                break;
                            }
                        }
    
                        tx.send(Event::Watermark(w)).await?;
                    },
                    Event::Snapshot(id) => {
                        tx.send(Event::Snapshot(id)).await?;
                    },
                    Event::Sentinel => {
                        while let Some(HeapEntry { event, .. }) = heap.pop() {
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