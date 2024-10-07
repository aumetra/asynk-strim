use divan::{black_box, black_box_drop, AllocProfiler};
use futures_lite::stream;
use std::pin::pin;

#[global_allocator]
static GLOBAL: AllocProfiler<mimalloc::MiMalloc> = AllocProfiler::new(mimalloc::MiMalloc);

const ITER_COUNT: usize = 1000;

#[divan::bench]
fn async_stream() {
    let stream = pin!(async_stream::stream!({
        for _ in 0..ITER_COUNT {
            yield black_box(1312);
        }
    }));

    stream::block_on(stream).for_each(black_box_drop);
}

#[divan::bench]
fn async_fn_stream() {
    let stream = pin!(async_fn_stream::fn_stream(|emitter| async move {
        for _ in 0..ITER_COUNT {
            emitter.emit(black_box(1312)).await;
        }
    }));

    stream::block_on(stream).for_each(black_box_drop);
}

#[divan::bench]
fn asynk_strim() {
    let stream = pin!(asynk_strim::strim_fn(|mut yielder| async move {
        for _ in 0..ITER_COUNT {
            yielder.yield_item(black_box(1312)).await;
        }
    }));

    stream::block_on(stream).for_each(black_box_drop);
}

fn main() {
    divan::main();
}
