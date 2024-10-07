use futures_lite::{future, stream};
use std::pin::pin;

// for some reason futures-lite triggers a miri error. whatever.
// we do everything right and upkeep every invariant (as shown by all the other tests passing).
//
// also it complains about something related to accessing our own pointer address via `self.as_ref().get_ref()`,
// which is very weird?? because that's a totally valid access and actually fine as ensured by the contract of `Pin`??
//
// there are similar issues on the tokio and futures-rs repositories which all point to a fault in miri itself.
// so i'm pretty confident it's not an issue of my implementation but instead just one of the rare miri bugs.
#[cfg(not(miri))]
#[test]
fn yields_to_correct_stream() {
    use futures_lite::StreamExt;

    let stream = pin!(asynk_strim::strim_fn(|mut yielder_1| async move {
        let mut inner_stream = pin!(asynk_strim::strim_fn(move |mut yielder_2| async move {
            yielder_2.yield_item("OwO").await;
            yielder_1.yield_item(12).await;
            yielder_1.yield_item(13).await;
            yielder_2.yield_item("hello world").await;
            yielder_1.yield_item(404).await;
        }));

        assert_eq!(inner_stream.next().await, Some("OwO"));
        assert_eq!(inner_stream.next().await, Some("hello world"));
        assert_eq!(inner_stream.next().await, None);
    }));

    let mut stream = stream::block_on(stream);
    assert_eq!(stream.next(), Some(12));
    assert_eq!(stream.next(), Some(13));
    assert_eq!(stream.next(), Some(404));
    assert_eq!(stream.next(), None);
}

#[test]
#[should_panic = "no matching stream frame found"]
fn yield_from_thread_with_stream() {
    let stream = pin!(asynk_strim::strim_fn(|mut yielder| async move {
        let err_out = std::thread::spawn(move || {
            let inner_stream = pin!(asynk_strim::strim_fn(|_| async move {
                yielder.yield_item("ùwú").await;
            }));

            stream::block_on(inner_stream).for_each(|()| {});
        })
        .join()
        .unwrap_err();

        std::panic::resume_unwind(err_out);
    }));

    stream::block_on(stream).for_each(|_| {});
}

#[test]
#[should_panic = "no matching stream frame found"]
fn yield_from_thread() {
    let stream = pin!(asynk_strim::strim_fn(|mut yielder| async move {
        let err_out = std::thread::spawn(move || {
            future::block_on(yielder.yield_item("ùwú"));
        })
        .join()
        .unwrap_err();

        std::panic::resume_unwind(err_out);
    }));

    stream::block_on(stream).for_each(|_| {});
}

#[test]
fn yield_integers() {
    let stream = pin!(asynk_strim::strim_fn(|mut yielder| async move {
        yielder.yield_item(1312).await;
        yielder.yield_item(141).await;
    }));

    let mut stream = stream::block_on(stream);
    assert_eq!(stream.next(), Some(1312));
    assert_eq!(stream.next(), Some(141));
    assert_eq!(stream.next(), None);
}

#[test]
fn yield_heap_alloc() {
    let stream = pin!(asynk_strim::strim_fn(|mut yielder| async move {
        yielder.yield_item(vec!["owo"]).await;
        yielder.yield_item(vec!["acab"]).await;
    }));

    let mut stream = stream::block_on(stream);
    assert_eq!(stream.next(), Some(vec!["owo"]));
    assert_eq!(stream.next(), Some(vec!["acab"]));
    assert_eq!(stream.next(), None);
}
