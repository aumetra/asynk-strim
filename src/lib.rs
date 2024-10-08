#![no_std]
#![doc = include_str!("../README.md")]
#![forbid(rust_2018_idioms)]
#![deny(missing_docs, unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(forbidden_lint_groups)]

use core::{future::Future, pin::pin, task};
use futures_core::Stream;

mod stream;
mod try_stream;
mod try_yielder;
mod waker;
mod yielder;

pub use self::try_yielder::TryYielder;
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
///
/// # Example
///
/// Let's yield some lyrics (Song: "Verdächtig" by Systemabsturz):
///
/// ```
/// # use futures_lite::StreamExt;
/// # use std::pin::pin;
/// # futures_lite::future::block_on(async {
/// let stream = asynk_strim::stream_fn(|mut yielder| async move {
///    yielder.yield_item("Fahr den Imsi-Catcher hoch").await;
///    yielder.yield_item("Mach das Richtmikro an").await;
///    yielder.yield_item("Bring Alexa auf den Markt").await;
///    yielder.yield_item("Zapf den Netzknoten an").await;
///    yielder.yield_item("Fahr den Ü-Wagen vor").await;
///    yielder.yield_item("Kauf den Staatstrojaner ein").await;
///    yielder.yield_item("Fake die Exit-Nodes bei Tor").await;
///    yielder.yield_item("Ihr wollt doch alle sicher sein").await;
/// });
///
/// let mut stream = pin!(stream);
/// while let Some(item) = stream.next().await {
///    println!("{item}");
/// }
/// # });
#[inline]
pub fn stream_fn<F, Item, Fut>(func: F) -> impl Stream<Item = Item>
where
    F: FnOnce(Yielder<Item>) -> Fut,
    Fut: Future<Output = ()>,
{
    crate::stream::init(func)
}

/// Jokey alias for [`stream_fn`]
///
/// For more elaborate documentation, see [`stream_fn`]
#[inline]
pub fn strim_fn<F, Item, Fut>(func: F) -> impl Stream<Item = Item>
where
    F: FnOnce(Yielder<Item>) -> Fut,
    Fut: Future<Output = ()>,
{
    stream_fn(func)
}

/// Create a new try stream
///
/// # Example
///
/// Let's yield some lyrics (Song: "Verdächtig" by Systemabsturz):
///
/// ```
/// # use futures_lite::StreamExt;
/// # use std::pin::pin;
/// # use std::convert::Infallible;
/// # futures_lite::future::block_on(async {
/// let stream = asynk_strim::try_stream_fn(|mut yielder| async move {
///   yielder.yield_ok("Fahr den Imsi-Catcher hoch").await;
///   yielder.yield_ok("Mach das Richtmikro an").await;
///   yielder.yield_ok("Bring Alexa auf den Markt").await;
///   yielder.yield_ok("Zapf den Netzknoten an").await;
///   yielder.yield_ok("Fahr den Ü-Wagen vor").await;
///   yielder.yield_ok("Kauf den Staatstrojaner ein").await;
///   yielder.yield_ok("Fake die Exit-Nodes bei Tor").await;
///   yielder.yield_ok("Ihr wollt doch alle sicher sein").await;
///
///   Ok::<_, Infallible>(())
/// });
///
/// let mut stream = pin!(stream);
/// while let Some(item) = stream.next().await {
///   println!("{item:?}");
/// }
/// # });
/// ```
#[inline]
pub fn try_stream_fn<F, Ok, Error, Fut>(func: F) -> impl Stream<Item = Result<Ok, Error>>
where
    F: FnOnce(TryYielder<Ok, Error>) -> Fut,
    Fut: Future<Output = Result<(), Error>>,
{
    crate::try_stream::init(func)
}

/// Jokey alias for [`try_stream_fn`]
///
/// For more elaborate documentation, see [`try_stream_fn`]
#[inline]
pub fn try_strim_fn<F, Ok, Error, Fut>(func: F) -> impl Stream<Item = Result<Ok, Error>>
where
    F: FnOnce(TryYielder<Ok, Error>) -> Fut,
    Fut: Future<Output = Result<(), Error>>,
{
    try_stream_fn(func)
}
