#[cfg(not(miri))]
#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
