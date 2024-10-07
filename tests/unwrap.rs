use futures_lite::{future, stream::StreamExt};
use std::{
    future::Future,
    mem::ManuallyDrop,
    pin::pin,
    ptr,
    task::{self, Poll, RawWaker, RawWakerVTable, Waker},
};

static WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    |data| unsafe {
        let data = data.cast::<Data<'_>>().as_ref().unwrap();
        let cloned = ManuallyDrop::new(data.waker.clone());
        RawWaker::new(cloned.data(), cloned.vtable())
    },
    |_| unreachable!(),
    |data| unsafe {
        let data = data.cast::<Data<'_>>().as_ref().unwrap();
        data.waker.wake_by_ref();
    },
    |_| unreachable!(),
);

struct Data<'a> {
    waker: &'a Waker,
}

#[derive(Debug, PartialEq)]
enum WrappedState {
    Accessible,
    Inaccessible,
}

async fn wrap_waker<Fut>(fut: Fut) -> Fut::Output
where
    Fut: Future,
{
    let mut fut = pin!(fut);
    future::poll_fn(|cx| {
        let data = Data { waker: cx.waker() };

        let waker = unsafe { Waker::new(ptr::from_ref(&data).cast(), &WAKER_VTABLE) };
        let waker = ManuallyDrop::new(waker);
        let mut cx = task::Context::from_waker(&waker);

        fut.as_mut().poll(&mut cx)
    })
    .await
}

fn is_wrapped() -> impl Future<Output = WrappedState> {
    future::poll_fn(|cx| {
        Poll::Ready(if *cx.waker().vtable() == WAKER_VTABLE {
            WrappedState::Accessible
        } else {
            WrappedState::Inaccessible
        })
    })
}

#[test]
fn unwrap_waker() {
    let mut stream = pin!(asynk_strim::strim_fn(|mut yielder| async move {
        // We shouldn't have access to our waker since it's wrapped in the `asynk_strim` wrapper
        yielder.yield_item(is_wrapped().await).await;

        // Here we should have access since the function explicitly unwraps it
        yielder
            .yield_item(asynk_strim::unwrap_waker(is_wrapped()).await)
            .await;
    }));

    future::block_on(wrap_waker(async move {
        assert_eq!(stream.next().await, Some(WrappedState::Inaccessible));
        assert_eq!(stream.next().await, Some(WrappedState::Accessible));
        assert_eq!(stream.next().await, None);
    }));
}
