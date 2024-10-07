# asynk-strim

Like `async-stream` but without macros. Like `async-fn-stream` but a little more efficient.

Features:

- macroless API
- one dependency (besides `futures-core` which I don't count since it provides the `Stream` definition)
- `no_std`-compatible, zero allocations

> [!IMPORTANT]  
> This crate adds a wrapper around the wakers that contains data and pointers needed to yield items.
> Crates like [`embassy`](https://embassy.dev) use a similar approach and will therefore clash with us.
>
> If you run into this issue (which will manifest as a runtime panic), you can use the `unwrap_waker` function.
> This function will wrap a future and remove the waker wrapper.
>
> While you can't use the yielder inside the unwrapped future, stuff like `embassy` should work again.

## Comparisons

### `async-stream`

In comparison to `async-stream` we offer the following advantages:

- no macros
- slightly faster performance
- `no_std` support

### `async-fn-stream`

In comparison to `async-stream` we offer the following advantages:

- no allocations
- slightly faster performance
- `no_std` support

## Acknowledgements

This crate combines approaches from the following crates:

- [`async-stream`][async-stream]
- [`async-fn-stream`][async-fn-stream]
- The PR by Sabrina Jewson adding a [function-based API to `async-stream`][sabrina-pr]
- The experimental PR by Hyeonu Park using [the waker-based approach][hyeonu-pr]

## License

Licensed under tither the MIT or Apache 2.0 license (at your choosing)

[async-stream]: https://github.com/tokio-rs/async-stream
[async-fn-stream]: https://github.com/dmitryvk/async-fn-stream
[sabrina-pr]: https://github.com/tokio-rs/async-stream/pull/74
[hyeonu-pr]: https://github.com/tokio-rs/async-stream/pull/105
