#[test]
fn echo_foo_test() {
    println!("echo_foo_test");
}

#[test]
fn in_source_dir() {
    let path = std::path::Path::new("./snowflake.txt");
    assert!(path.exists())
}
