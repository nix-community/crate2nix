#![feature(no_core)]
#![no_core]

#![cfg_attr(not(target_os = "none"), feature(rustc_attrs))]

#[cfg(not(target_os = "none"))]
#[rustc_builtin_macro]
macro_rules! compile_error {
    ($msg:expr $(,)?) => {{ /* compiler built-in */ }};
}

#[cfg(not(target_os = "none"))]
compile_error!{ "Didn't cross compile!" }
