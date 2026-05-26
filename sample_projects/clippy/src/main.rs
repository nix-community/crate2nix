/// a lousy implementation of cat
fn main() {
    assert!(true);
    clippy::run();
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("expected one argument");
    } else {
        let path = std::path::PathBuf::from(&args[1]);
        let content = std::fs::read_to_string(path).unwrap();
        print!("{}", content);
    }
}

#[test]
fn no_cfg_test() {
    assert!(true);
}

#[cfg(test)]
mod test {
    #[test]
    fn with_cfg_test() {
        assert!(true);
    }
}
