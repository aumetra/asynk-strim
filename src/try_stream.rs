use crate::{try_yielder::TryYielder, yielder::Yielder};
use core::{
    future::Future,
    hint::unreachable_unchecked,
    marker::PhantomData,
    pin::Pin,
    ptr,
    task::{self, Poll},
};
use futures_core::{FusedStream, Stream};
use pin_project_lite::pin_project;

#[inline]
pub fn init<F, Fut, Ok, Error>(func: F) -> impl Stream<Item = Result<Ok, Error>>
where
    F: FnOnce(TryYielder<Ok, Error>) -> Fut,
    Fut: Future<Output = Result<(), Error>>,
{
    AsynkStrim::Initial { func: Some(func) }
}

pin_project! {
    /// IMPORTANT: Never EVER EVER create this stream in the state `Initial` with the `func` parameter set to `None`
    /// Doing this will trigger undefined behaviour.
    ///
    /// IMPORTANT: Never EVER EVER construct a stream in the `MarkerStuff` state.
    /// Doing this will trigger undefined behaviour.
    #[project = AsynkStrimProj]
    #[project(!Unpin)]
    enum AsynkStrim<F, Fut, Item> {
        Initial {
            func: Option<F>,
        },
        Progress {
            #[pin]
            fut: Fut,
        },
        Done,
        MarkerStuff {
            _item: PhantomData<Item>,
        }
    }
}

impl<F, Fut, Ok, Error> Stream for AsynkStrim<F, Fut, Result<Ok, Error>>
where
    F: FnOnce(TryYielder<Ok, Error>) -> Fut,
    Fut: Future<Output = Result<(), Error>>,
{
    type Item = Result<Ok, Error>;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let stream_address = ptr::from_ref(self.as_ref().get_ref()) as usize;
        loop {
            match self.as_mut().project() {
                AsynkStrimProj::Initial { func } => {
                    // at the end of the function we transition into the progress state.
                    // this state is never initialized with `func` set to `None`.
                    //
                    // we actually only do this to be able to use `.take()` to remove the function from the future.
                    #[allow(unsafe_code)]
                    let func = unsafe { func.take().unwrap_unchecked() };
                    let fut = func(TryYielder::new(Yielder::new(stream_address)));

                    self.set(Self::Progress { fut });
                }
                AsynkStrimProj::Progress { fut, .. } => {
                    let mut out = None;
                    let poll_output =
                        crate::waker::with_context(cx.waker(), stream_address, &mut out, |cx| {
                            fut.poll(cx)
                        });

                    match (poll_output, out) {
                        (Poll::Ready(result), ..) => {
                            self.set(AsynkStrim::Done);
                            if let Err(err) = result {
                                break Poll::Ready(Some(Err(err)));
                            }
                        }
                        (Poll::Pending, Some(item)) => break Poll::Ready(Some(item)),
                        (Poll::Pending, None) => break Poll::Pending,
                    }
                }
                AsynkStrimProj::Done => break Poll::Ready(None),
                AsynkStrimProj::MarkerStuff { .. } => {
                    // the state machine will never enter this state.
                    // documented on the state machine level.
                    #[allow(unsafe_code)]
                    unsafe {
                        unreachable_unchecked()
                    }
                }
            }
        }
    }
}

impl<F, Fut, Ok, Error> FusedStream for AsynkStrim<F, Fut, Result<Ok, Error>>
where
    F: FnOnce(TryYielder<Ok, Error>) -> Fut,
    Fut: Future<Output = Result<(), Error>>,
{
    #[inline]
    fn is_terminated(&self) -> bool {
        matches!(self, Self::Done)
    }
}
