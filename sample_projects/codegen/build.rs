// src/build.rs
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use dbus_codegen;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("hello.rs");
    let mut f = File::create(&dest_path).unwrap();

    f.write_all(
        b"
        pub fn message() -> &'static str {
            \"Hello, World!\"
        }
    ",
    )
    .unwrap();
}