#[test_case]
fn vga_println_test() {
    let test_str = "This is a test hahahaha!";
    println!("{}", test_str);
    for _ in 0..200 {
        print!("a");
    }

    for _ in 0..200 {
        println!("This is a test line!");
    }
}
