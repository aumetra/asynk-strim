use futures_lite::stream;
use std::pin::pin;

#[test]
fn exits_early() {
    let stream = pin!(asynk_strim::try_stream_fn(|mut yielder| async move {
        yielder.yield_ok(42).await;
        return Err("oh no");

        #[allow(unreachable_code)]
        yielder.yield_ok(1337).await;

        Ok(())
    }));

    let mut stream = stream::block_on(stream);
    assert_eq!(stream.next(), Some(Ok(42)));
    assert_eq!(stream.next(), Some(Err("oh no")));
    assert_eq!(stream.next(), None);
}
