#[test]
fn echo_foo_test() {
    println!("echo_foo_test");
}

#[test]
fn in_source_dir() {
    let path = std::path::Path::new("./snowflake.txt");
    assert!(path.exists())
}

#[test]
fn exec_cowsay() {
    std::process::Command::new("cowsay").spawn().unwrap();
}
