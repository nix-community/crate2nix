fn main() {
    #[cfg(target_family = "unix")]
    hello_world_lib::hello_world("bin_with_lib_dep");
}
