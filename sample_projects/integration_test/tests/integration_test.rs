use cli_test_dir::*;

#[test]
fn read_source_file() {
    let testdir = TestDir::new("integration_test", "temp");
    let source_file = testdir.src_path("tests/hello");
    let output = testdir.cmd()
        .arg(source_file)
        .output()
        .expect_success();
    let string = String::from_utf8_lossy(&output.stdout);
    assert_eq!(string, "hello world\n");
}

#[test]
fn write_output_file() {
    let testdir = TestDir::new("integration_test", "temp");
    testdir.create_file("input.txt", "yay");
    let output = testdir.cmd()
        .arg("input.txt")
        .output()
        .expect_success();
    let string = String::from_utf8_lossy(&output.stdout);
    assert_eq!(string, "yay");
}
