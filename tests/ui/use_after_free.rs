use futures_lite::StreamExt;
use std::pin::pin;

fn main() {
    futures_lite::future::block_on(async move {
        let mut outer = vec![];
        {
            let v = vec![0; 10];
            let v_ref = &v;
            let mut s = pin!(asynk_strim::stream_fn(|mut yielder| async move {
                for x in v_ref {
                    yielder.yield_item(x).await;
                }
            }));

            while let Some(x) = s.next().await {
                outer.push(x);
            }
        };
        // use-after-free
        println!("{outer:?}");
    });
}
