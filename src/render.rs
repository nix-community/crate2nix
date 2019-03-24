use std::fs::File;
use std::io::Write;
use std::path::Path;

use failure::format_err;
use failure::Error;
use lazy_static::lazy_static;
use tera::Tera;

use crate::DefaultNix;

pub fn default_nix(metadata: &DefaultNix) -> Result<String, Error> {
    Ok(TERA
        .render_value("default.nix.tera", metadata)
        .map_err(|e| {
            format_err!(
                "while rendering default.nix: {:?}\nMetadata: {:?}",
                e,
                metadata
            )
        })?)
}

pub fn write_to_file(path: impl AsRef<Path>, contents: &str) -> Result<(), Error> {
    let mut output_file = File::create(&path)?;
    output_file.write_all(contents.as_bytes())?;
    println!(
        "Generated {} successfully.",
        path.as_ref().to_string_lossy()
    );
    Ok(())
}

lazy_static! {
    static ref TERA: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_template(
            "default.nix.tera",
            include_str!("../templates/default.nix.tera"),
        )
        .expect("while adding template");
        tera.autoescape_on(vec![".nix.tera", ".nix"]);
        tera.set_escape_fn(escape_nix_string);
        tera
    };
}

/// Escapes a string as a nix string.
///
/// ```
/// use cargo2nix::render::escape_nix_string;
/// assert_eq!("\"abc\"", escape_nix_string("abc"));
/// assert_eq!("\"a\\\"bc\"", escape_nix_string("a\"bc"));
/// assert_eq!("\"a$bc\"", escape_nix_string("a$bc"));
/// assert_eq!("\"a$\"", escape_nix_string("a$"));
/// assert_eq!("\"a\\${bc\"", escape_nix_string("a${bc"));
/// ```
pub fn escape_nix_string(raw_string: &str) -> String {
    let mut ret = String::with_capacity(raw_string.len() + 2);
    ret.push('"');
    let mut peekable_chars = raw_string.chars().peekable();
    while let Some(c) = peekable_chars.next() {
        if c == '\\' || c == '"' {
            ret.push('\\');
        } else if c == '$' && peekable_chars.peek() == Some(&'{') {
            ret.push('\\');
        }
        ret.push(c);
    }
    ret.push('"');
    ret
}
