fn main() {
    if shared::host_only() {
        println!("host-only leaked into the target build");
    } else {
        println!("target build without host-only");
    }
}
