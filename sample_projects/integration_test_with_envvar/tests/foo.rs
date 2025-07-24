#[test]
fn test_main() {
  let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_integration_test_with_envvar"));
  assert_eq!(&cmd.output().unwrap().stdout, b"main.rs\n");
}
#[test]
fn test_exe() {
  let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_exe"));
  assert_eq!(&cmd.output().unwrap().stdout, b"exe.rs\n");
}
#[test]
fn test_exe2() {
  let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_exe-with-dash"));
  assert_eq!(&cmd.output().unwrap().stdout, b"exe-with-dash.rs\n");
}
