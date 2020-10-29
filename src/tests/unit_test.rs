#[test_case]
fn trivial_assertion1() {
    serial_print!("unit test...");
    assert_eq!(1, 1);
    serial_print!("passed");
}
