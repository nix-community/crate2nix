/// a lousy implementation of cat
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("expected one argument");
    } else {
        let path = std::path::PathBuf::from(&args[1]);
        let content = std::fs::read_to_string(path).unwrap();
        print!("{}", content);
    }
}
