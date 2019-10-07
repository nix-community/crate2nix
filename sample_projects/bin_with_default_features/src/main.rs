#[cfg(feature = "use_lib")]
use renamed_hello_world_lib;

fn main() {
    #[cfg(feature = "do_not_activate")]
    let str = ", do_not_activate";
    #[cfg(not(feature = "do_not_activate"))]
    let str = "";
    #[cfg(feature = "use_lib")]
    renamed_hello_world_lib::hello_world(
        &format!("bin_with_default_features{}", str));
}
