use pmd_wan::Resolution;

fn main() {
    let resolution = Resolution::new(32, 32);
    println!("Hello from a git workspace with {}!", resolution.x);
}
