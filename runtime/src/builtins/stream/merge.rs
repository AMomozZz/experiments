use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::builtins::stream::Event;
use crate::builtins::time::Time;
use crate::runner::context::Context;
use crate::traits::Data;

use super::Stream;

#[derive(Debug)]
struct HeapEntry<T> {
    time: Time,
    seq: usize,
    event: Event<T>,
}

impl<T> Ord for HeapEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time.cmp(&self.time).then_with(|| other.seq.cmp(&self.seq))
    }
}
impl<T> PartialOrd for HeapEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> PartialEq for HeapEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.seq == other.seq
    }
}
impl<T> Eq for HeapEntry<T> {}

impl<T: Data> Stream<T> {
    pub fn merge(mut self, ctx: &mut Context, mut other: Self) -> Self {
        ctx.operator(|tx| async move {
            let mut l_done = false;
            let mut r_done = false;
            let mut l_watermark = Time::zero();
            let mut r_watermark = Time::zero();
            loop {
                tokio::select! {
                    event = self.recv(), if !l_done => match event {
                        Event::Data(t, v) => tx.send(Event::Data(t, v)).await?,
                        Event::Watermark(t) => {
                            if t < r_watermark {
                                tx.send(Event::Watermark(t)).await?;
                            } else if l_watermark < r_watermark && r_watermark < t {
                                tx.send(Event::Watermark(r_watermark)).await?;
                            }
                            l_watermark = t;
                        },
                        Event::Sentinel => {
                            l_done = true;
                            if r_done {
                                tx.send(Event::Sentinel).await?;
                                break;
                            }
                        },
                        Event::Snapshot(i) => tx.send(Event::Snapshot(i)).await?
                    },
                    event = other.recv(), if !r_done => match event {
                        Event::Data(t, v) => tx.send(Event::Data(t, v)).await?,
                        Event::Watermark(t) => {
                            if t < l_watermark {
                                tx.send(Event::Watermark(t)).await?;
                            } else if r_watermark < l_watermark && l_watermark < t {
                                tx.send(Event::Watermark(l_watermark)).await?;
                            }
                            r_watermark = t;
                        },
                        Event::Sentinel => {
                            r_done = true;
                            if l_done {
                                tx.send(Event::Sentinel).await?;
                                break;
                            }
                        },
                        Event::Snapshot(i) => tx.send(Event::Snapshot(i)).await?
                    },
                };
            }
            Ok(())
        })
    }

    pub fn sorted_merge(mut self, ctx: &mut Context, mut other: Self) -> Self {
        ctx.operator(|tx| async move {
            let mut l_done = false;
            let mut r_done = false;
            let mut l_watermark = Time::zero();
            let mut r_watermark = Time::zero();

            let mut buffer = BinaryHeap::<HeapEntry<T>>::new();
            let mut seq = 0;

            loop {
                tokio::select! {
                    event = self.recv(), if !l_done => match event {
                        Event::Data(t, v) => {
                            buffer.push(HeapEntry { time: t, seq, event: Event::Data(t, v) });
                            seq += 1;
                        },
                        Event::Watermark(t) => l_watermark = t,
                        Event::Sentinel => l_done = true,
                        Event::Snapshot(i) => tx.send(Event::Snapshot(i)).await?,
                    },

                    event = other.recv(), if !r_done => match event {
                        Event::Data(t, v) => {
                            buffer.push(HeapEntry { time: t, seq, event: Event::Data(t, v) });
                            seq += 1;
                        },
                        Event::Watermark(t) => r_watermark = t,
                        Event::Sentinel => r_done = true,
                        Event::Snapshot(i) => tx.send(Event::Snapshot(i)).await?,
                    },

                    else => {println!("{:?}",buffer.peek())},
                }

                let watermark_min = l_watermark.min(r_watermark);
                while let Some(top) = buffer.peek() {
                    if top.time <= watermark_min || (l_done && r_done)  {
                        let HeapEntry { event, .. } = buffer.pop().unwrap();
                        tx.send(event).await?;
                    } else {
                        break;
                    }
                }

                if l_watermark <= r_watermark && r_watermark != Time::zero() {
                    tx.send(Event::Watermark(l_watermark)).await?;
                } else if r_watermark < l_watermark && l_watermark != Time::zero() {
                    tx.send(Event::Watermark(r_watermark)).await?;
                }

                if l_done && r_done && buffer.is_empty() {
                    tx.send(Event::Sentinel).await?;
                    break;
                }
            }

            Ok(())
        })
    }
}
