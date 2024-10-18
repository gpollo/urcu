#[test]
fn hashmap_lifetime() {
    let tests = trybuild::TestCases::new();

    tests.pass("tests/ui/hashmap/get-ok-0.rs");
    tests.pass("tests/ui/hashmap/get-ok-1.rs");
    tests.compile_fail("tests/ui/hashmap/get-fail-0.rs");
    tests.compile_fail("tests/ui/hashmap/get-fail-1.rs");

    tests.pass("tests/ui/hashmap/iter-ok-0.rs");
    tests.pass("tests/ui/hashmap/iter-ok-1.rs");
    tests.compile_fail("tests/ui/hashmap/iter-fail-0.rs");
    tests.compile_fail("tests/ui/hashmap/iter-fail-1.rs");
}

#[test]
fn list_lifetime() {
    let tests = trybuild::TestCases::new();

    tests.pass("tests/ui/list/back-ok-0.rs");
    tests.pass("tests/ui/list/back-ok-1.rs");
    tests.compile_fail("tests/ui/list/back-fail-0.rs");
    tests.compile_fail("tests/ui/list/back-fail-1.rs");

    tests.pass("tests/ui/list/front-ok-0.rs");
    tests.pass("tests/ui/list/front-ok-1.rs");
    tests.compile_fail("tests/ui/list/front-fail-0.rs");
    tests.compile_fail("tests/ui/list/front-fail-1.rs");

    tests.pass("tests/ui/list/iter-forward-ok-0.rs");
    tests.pass("tests/ui/list/iter-forward-ok-1.rs");
    tests.compile_fail("tests/ui/list/iter-forward-fail-0.rs");
    tests.compile_fail("tests/ui/list/iter-forward-fail-1.rs");

    tests.pass("tests/ui/list/iter-reverse-ok-0.rs");
    tests.pass("tests/ui/list/iter-reverse-ok-1.rs");
    tests.compile_fail("tests/ui/list/iter-reverse-fail-0.rs");
    tests.compile_fail("tests/ui/list/iter-reverse-fail-1.rs");
}

#[test]
fn stack_lifetime() {
    let tests = trybuild::TestCases::new();

    tests.pass("tests/ui/stack/peek-ok-0.rs");
    tests.pass("tests/ui/stack/peek-ok-1.rs");
    tests.compile_fail("tests/ui/stack/peek-fail-0.rs");
    tests.compile_fail("tests/ui/stack/peek-fail-1.rs");

    tests.pass("tests/ui/stack/iter-ok-0.rs");
    tests.pass("tests/ui/stack/iter-ok-1.rs");
    tests.compile_fail("tests/ui/stack/iter-fail-0.rs");
    tests.compile_fail("tests/ui/stack/iter-fail-1.rs");
}
