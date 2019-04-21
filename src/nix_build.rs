//! Code for invoking nix_build.

use std::io::BufRead;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use failure::bail;
use failure::format_err;
use failure::Error;

/// Call `nix build` in the given directory on the `default.nix` in that directory.
pub fn nix_build(project_dir: impl AsRef<Path>, nix_attr: &str) -> Result<(), Error> {
    let project_dir = project_dir.as_ref().to_string_lossy().to_string();
    println!("Building {}.", project_dir);
    let status = Command::new("nix")
        .current_dir(&project_dir)
        .args(&["--show-trace", "build", "-f", "default.nix", nix_attr])
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format_err!("while spawning nix-build for {}: {}", project_dir, e))?;
    //    let default_nix = PathBuf::from(&project_dir).join("default.nix");
    //    dump_with_lines(&default_nix)?;
    if !status.success() {
        let default_nix = PathBuf::from(&project_dir).join("default.nix");
        dump_with_lines(&default_nix)?;
        bail!(
            "nix-build {}\n=> exited with: {}",
            project_dir,
            status.code().unwrap_or(-1)
        );
    }
    println!("Built {} successfully.", project_dir);

    Ok(())
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

    Ok(String::from_utf8(output.stdout)
        .map_err(|_e| format_err!("output of {} is not UTF8!", cmd_path))?)
}
