fn main() {
    let _stream = asynk_strim::strim_fn(|mut yielder| async move {
        let _first = yielder.yield_item(1312);
        let _second = yielder.yield_item(141);
    });
}