#![no_std]
#![doc = include_str!("../README.md")]
#![forbid(rust_2018_idioms)]
#![deny(missing_docs, unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(forbidden_lint_groups)]

use core::{future::Future, pin::pin, task};
use futures_core::Stream;

mod stream;
mod waker;
mod yielder;

pub use self::yielder::Yielder;

/// Unwrap the waker
///
/// This is an escape hatch if a library depends on a similar hack we do,
/// where we wrap the waker in a struct to store additional data.
///
/// An example is the [`embassy`](https://embassy.dev/) crate.
///
/// # Panics
///
/// The future will panic if the waker is not found.
/// This happens if you use this function outside of the context of a stream generator.
#[inline]
pub async fn unwrap_waker<Fut>(future: Fut) -> Fut::Output
where
    Fut: Future,
{
    let mut future = pin!(future);
    core::future::poll_fn(|cx| {
        let unwrapped = crate::waker::unwrap_inner(cx.waker()).expect("waker not found");
        let mut cx = task::Context::from_waker(unwrapped);
        future.as_mut().poll(&mut cx)
    })
    .await
}

/// Create a new stream
#[inline]
pub fn strim_fn<F, Item, Fut>(func: F) -> impl Stream<Item = Item>
where
    F: FnOnce(Yielder<Item>) -> Fut,
    Fut: Future<Output = ()>,
{
    crate::stream::init(func)
}
