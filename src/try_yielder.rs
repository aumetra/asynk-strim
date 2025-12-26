use crate::yielder::Yielder;

/// Handle to allow you to yield something from the stream
pub struct TryYielder<Ok, Error> {
    yielder: Yielder<Result<Ok, Error>>,
}

impl<Ok, Error> TryYielder<Ok, Error> {
    /// Yield a success value from the stream
    #[inline]
    pub async fn yield_ok(&mut self, item: Ok) {
        self.yielder.yield_item(Ok(item)).await;
    }

    /// Yield an error value from the stream
    #[inline]
    pub async fn yield_error(&mut self, item: Error) {
        self.yielder.yield_item(Err(item)).await;
    }

    /// # Interntionally kept private
    ///
    /// Check the `internal_clone` impl on `Yielder` for more info.
    pub(crate) fn internal_clone(&self) -> Self {
        Self {
            yielder: self.yielder.internal_clone(),
        }
    }
}

#[doc(hidden)]
impl<Ok, Error> From<Yielder<Result<Ok, Error>>> for TryYielder<Ok, Error> {
    #[inline]
    fn from(yielder: Yielder<Result<Ok, Error>>) -> Self {
        Self { yielder }
    }
}
