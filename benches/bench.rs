use criterion::{criterion_group, criterion_main, Criterion};
use futures_lite::{stream, Stream};
use std::{hint::black_box, pin::pin};

const ITER_COUNT: usize = 1000;

fn consume_stream<S: Stream + Unpin>(stream: S) {
    stream::block_on(stream).for_each(|item| black_box(drop)(item))
}

fn async_stream(c: &mut Criterion) {
    c.bench_function("async_stream", |b| {
        b.iter(|| {
            let stream = pin!(async_stream::stream!({
                for _ in 0..ITER_COUNT {
                    yield black_box(1312);
                }
            }));

            consume_stream(stream);
        })
    });
}

fn async_fn_stream(c: &mut Criterion) {
    c.bench_function("async_fn_stream", |b| {
        b.iter(|| {
            let stream = pin!(async_fn_stream::fn_stream(|emitter| async move {
                for _ in 0..ITER_COUNT {
                    emitter.emit(black_box(1312)).await;
                }
            }));

            consume_stream(stream);
        })
    });
}

fn asynk_strim(c: &mut Criterion) {
    c.bench_function("asynk_strim", |b| {
        b.iter(|| {
            let stream = pin!(asynk_strim::strim_fn(|mut yielder| async move {
                for _ in 0..ITER_COUNT {
                    yielder.yield_item(black_box(1312)).await;
                }
            }));

            consume_stream(stream);
        })
    });
}

criterion_group!(benches, async_stream, async_fn_stream, asynk_strim);
criterion_main!(benches);
