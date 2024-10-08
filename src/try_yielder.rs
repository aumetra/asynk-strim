use crate::yielder::Yielder;

/// Handle to allow you to yield something from the stream
pub struct TryYielder<Ok, Error> {
    yielder: Yielder<Result<Ok, Error>>,
}

impl<Ok, Error> TryYielder<Ok, Error> {
    #[inline]
    pub(crate) fn new(yielder: Yielder<Result<Ok, Error>>) -> Self {
        Self { yielder }
    }

    /// Yield a result from the stream
    #[inline]
    pub async fn yield_result(&mut self, item: Result<Ok, Error>) {
        self.yielder.yield_item(item).await;
    }

    /// Yield a success value from the stream
    #[inline]
    pub async fn yield_ok(&mut self, item: Ok) {
        self.yield_result(Ok(item)).await;
    }

    /// Yield an error value from the stream
    #[inline]
    pub async fn yield_error(&mut self, item: Error) {
        self.yield_result(Err(item)).await;
    }
}
