use core::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{self, Poll},
};

struct YieldFuture<Item> {
    item: Option<Item>,
    stream_address: usize,
}

impl<Item> Future for YieldFuture<Item> {
    type Output = ();

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        if self.item.is_none() {
            return Poll::Ready(());
        }

        let mut frame = crate::waker::find_frame(cx.waker());
        let out_ptr = loop {
            let curr_frame = frame.expect("no matching stream frame found");
            if curr_frame.address == self.stream_address {
                let mut out_ptr = curr_frame.out_ref.cast::<Option<Item>>();

                // the pointer is _always_ initialized to `None`.
                #[allow(unsafe_code)]
                break unsafe { out_ptr.as_mut() };
            }

            frame = {
                // we always set this to a valid pointer to an option
                #[allow(unsafe_code)]
                unsafe {
                    *curr_frame.prev.as_ref()
                }
            };
        };

        debug_assert!(out_ptr.is_none(), "double yield. slow down, bestie");
        *out_ptr = self.item.take();

        Poll::Pending
    }
}

impl<Item> Unpin for YieldFuture<Item> {}

/// Handle to allow you to yield something from the stream
pub struct Yielder<Item> {
    _marker: PhantomData<fn(Item) -> Item>,
    stream_address: usize,
}

impl<Item> Yielder<Item> {
    #[inline]
    pub(crate) fn new(stream_address: usize) -> Self {
        Self {
            _marker: PhantomData,
            stream_address,
        }
    }

    /// Yield an item from the stream
    #[inline]
    pub async fn yield_item(&mut self, item: Item) {
        let future = YieldFuture {
            item: Some(item),
            stream_address: self.stream_address,
        };

        future.await;
    }

    /// # Intentionally internal
    ///
    /// Double yields are not well-defined by the crate and any item may be lost.
    /// Hence the yielder is not cloneable by the user to avoid misuse.
    ///
    /// This function is strictly required so we can yield in the `try_stream` implementation
    /// after the inner future terminated with an error.
    pub(crate) fn internal_clone(&self) -> Self {
        Self {
            _marker: self._marker,
            stream_address: self.stream_address,
        }
    }
}
