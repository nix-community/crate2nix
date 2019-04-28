use std::io::Write;
use std::process::Command;

use failure::bail;
use failure::format_err;
use failure::Error;
use serde_json::Value;

#[test]
fn nix_tests() {
    let result = run_instantiate().expect("while running instantiate");
    let json_value: Value = serde_json::from_str(&result).expect("while reading result as json");
    let result = serde_json::to_string_pretty(&json_value).expect("while pretty printing");
    if result != "\"OK\"" {
        panic!("Results with failures:\n{}", result);
    }
}

pub fn run_instantiate() -> Result<String, Error> {
    let output = Command::new("nix-instantiate")
        .args(&[
            "--eval",
            "--strict",
            "--json",
            "--show-trace",
            "./templates/nix/crate2nix/tests/default.nix",
        ])
        .output()
        .map_err(|e| format_err!("while spawning nix-instantiate: {}", e))?;
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;
        bail!(
            "nix-instantiate\n=> exited with: {}",
            output.status.code().unwrap_or(-1)
        );
    }

    Ok(String::from_utf8(output.stdout)
        .map_err(|_e| format_err!("output of nix-instantiate is not UTF8!"))?)
}
