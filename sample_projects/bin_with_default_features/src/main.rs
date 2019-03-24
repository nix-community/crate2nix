#[cfg(feature = "use_lib")]
use hello_world_lib;

fn main() {
    #[cfg(feature = "use_lib")]
    hello_world_lib::hello_world("bin_with_default_features");
    #[cfg(feature = "do_not_activate")]
    println!("COMPILED with do_not_activate");
}
