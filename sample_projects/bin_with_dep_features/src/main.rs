#[cfg(feature = "use_lib")]
use renamed_hello_world_lib;

fn main() {
    #[cfg(feature = "use_lib")]
    renamed_hello_world_lib::hello_world("bin_with_dep_features");
}
