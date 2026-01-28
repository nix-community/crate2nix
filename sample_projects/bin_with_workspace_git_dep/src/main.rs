fn main() {
    let conf = nix_conf_parser::NixConf::parse_str("experimental-features = flakes nix-command")
        .expect("Failed to parse nix conf");
    let features = conf.get("experimental-features").unwrap();
    println!("Hello from bin_with_workspace_git_dep! features={features}");
}
