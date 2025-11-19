#[cfg(not(miri))]
#[test]
#[ignore = "only run on stable pls"]
fn tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
