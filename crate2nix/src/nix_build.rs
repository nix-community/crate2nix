//! Code for invoking `nix-build`.

use std::io::BufRead;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use anyhow::bail;
use anyhow::format_err;
use anyhow::Error;

/// Call `nix build` in the given directory on the `default.nix` in that directory.
pub fn nix_build(
    project_dir: impl AsRef<Path>,
    nix_attr: &str,
    features: &[&str],
) -> Result<(), Error> {
    let project_dir_path = project_dir.as_ref();
    let project_dir = project_dir_path.to_string_lossy().to_string();

    let result = crate::command::run(
        &format!("Building {}", project_dir),
        Command::new("nix")
            .current_dir(&project_dir)
            .args([
                "--show-trace",
                "build",
                "-f",
                "default.nix",
                nix_attr,
                "--arg",
                "rootFeatures",
            ])
            .arg(format!(
                "[ {} ]",
                features
                    .iter()
                    .map(|s| crate::render::escape_nix_string(s))
                    .collect::<Vec<_>>()
                    .join(" ")
            )),
    );

    if result.is_err() {
        dump_with_lines(project_dir_path.join("default.nix"))?;
    }

    result
}

/// Dump the content of the specified file with line numbers to stdout.
pub fn dump_with_lines(file_path: impl AsRef<Path>) -> Result<(), Error> {
    let file_path = file_path.as_ref().to_string_lossy().to_string();
    let content = std::io::BufReader::new(std::fs::File::open(&file_path)?);
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    for (idx, line) in content.lines().enumerate() {
        writeln!(handle, "{:>5}: {}", idx + 1, line?)?;
    }

    Ok(())
}

/// Run the command at the given path without arguments and capture the output in the return value.
pub fn run_cmd(cmd_path: impl AsRef<Path>) -> Result<String, Error> {
    let cmd_path = cmd_path.as_ref().to_string_lossy().to_string();
    let output = Command::new(&cmd_path)
        .output()
        .map_err(|e| format_err!("while spawning {}: {}", cmd_path, e))?;
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;
        bail!(
            "{}\n=> exited with: {}",
            cmd_path,
            output.status.code().unwrap_or(-1)
        );
    }

    String::from_utf8(output.stdout)
        .map_err(|_e| format_err!("output of {} is not UTF8!", cmd_path))
}
