pub fn main() {
    compile_error!("This binary shouldn’t be compiled, as it depend on \"afeature\", which souldn't be enabled!");
}