//| meta: label
fn somefunc(a: i64) {
    //| solution <<
    return 2 * a;
    //| placeholder
    // jkl
    //| >>
}

fn test() {
    assert!(somefunc(5) == 10);
    assert!(somefunc(10) == 20);
}
