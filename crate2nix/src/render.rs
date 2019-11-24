//! "Render" files using tera templates.
use std::fs::File;
use std::io::Write;
use std::path::Path;

use failure::format_err;
use failure::Error;
use lazy_static::lazy_static;
use tera::{Tera, Context};

use crate::target_cfg::{Cfg, CfgExpr};
use crate::BuildInfo;
use std::collections::HashMap;
use std::str::FromStr;

pub fn render_build_file(metadata: &BuildInfo) -> Result<String, Error> {
    Ok(TERA.render("build.nix.tera", &Context::from_serialize(metadata)?).map_err(|e| {
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
        tera.add_raw_templates(vec![
            (
                "build.nix.tera",
                include_str!("../templates/build.nix.tera"),
            ),
            (
                "nix/crate2nix/default.nix",
                include_str!("../templates/nix/crate2nix/default.nix"),
            ),
        ])
        .expect("while adding template");
        tera.autoescape_on(vec![".nix.tera", ".nix"]);
        tera.set_escape_fn(escape_nix_string);
        tera.register_filter("cfg_to_nix_expr", cfg_to_nix_expr_filter);
        tera
    };
}

fn cfg_to_nix_expr_filter(
    value: &tera::Value,
    _args: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    match value {
        tera::Value::String(key) => {
            if key.starts_with("cfg(") && key.ends_with(')') {
                let cfg = &key[4..key.len() - 1];

                let expr = CfgExpr::from_str(&cfg).map_err(|e| {
                    tera::Error::msg(format!(
                        "cfg_to_nix_expr_filter: Could not parse '{}': {}",
                        cfg, e
                    ))
                })?;
                Ok(tera::Value::String(cfg_to_nix_expr(&expr)))
            } else {
                // It is hopefully a target "triplet".
                let condition =
                    format!("(stdenv.hostPlatform.config == {})", escape_nix_string(key));
                Ok(tera::Value::String(condition))
            }
        }
        _ => Err(tera::Error::msg(format!(
            "cfg_to_nix_expr_filter: Expected string, got {:?}",
            value
        ))),
    }
}

/// Renders a config expression to nix code.
fn cfg_to_nix_expr(cfg: &CfgExpr) -> String {
    fn target(target_name: &str) -> String {
        escape_nix_string(if target_name.starts_with("target_") {
            &target_name[7..]
        } else {
            target_name
        })
    }

    fn render(result: &mut String, cfg: &CfgExpr) {
        match cfg {
            CfgExpr::Value(Cfg::Name(name)) => {
                result.push_str(&format!("target.{}", target(name)));
            }
            CfgExpr::Value(Cfg::KeyPair(key, value)) => {
                let escaped_value = escape_nix_string(value);
                result.push_str(&if key == "feature" {
                    format!(
                        "(builtins.elem {} resolvedDefaultFeatures)",
                        escaped_value,
                    )
                } else {
                    format!(
                        "(target.{} == {})",
                        target(key),
                        escaped_value,
                    )
                });
            }
            CfgExpr::Not(expr) => {
                result.push_str("(!");
                render(result, expr);
                result.push(')');
            }
            CfgExpr::All(expressions) => {
                result.push('(');
                render(result, &expressions[0]);
                for expr in &expressions[1..] {
                    result.push_str(" && ");
                    render(result, expr);
                }
                result.push(')');
            }
            CfgExpr::Any(expressions) => {
                result.push('(');
                render(result, &expressions[0]);
                for expr in &expressions[1..] {
                    result.push_str(" || ");
                    render(result, expr);
                }
                result.push(')');
            }
        }
    }

    let mut ret = String::new();
    render(&mut ret, cfg);
    ret
}

#[test]
fn test_render_cfg_to_nix_expr() {
    fn name(value: &str) -> CfgExpr {
        CfgExpr::Value(Cfg::Name(value.to_string()))
    }

    fn kv(key: &str, value: &str) -> CfgExpr {
        use crate::target_cfg::Cfg::KeyPair;
        CfgExpr::Value(KeyPair(key.to_string(), value.to_string()))
    }

    assert_eq!("target.\"unix\"", &cfg_to_nix_expr(&name("unix")));
    assert_eq!(
        "(target.\"os\" == \"linux\")",
        &cfg_to_nix_expr(&kv("target_os", "linux"))
    );
    assert_eq!(
        "(!(target.\"os\" == \"linux\"))",
        &cfg_to_nix_expr(&CfgExpr::Not(Box::new(kv("target_os", "linux"))))
    );
    assert_eq!(
        "(target.\"unix\" || (target.\"os\" == \"linux\"))",
        &cfg_to_nix_expr(&CfgExpr::Any(vec![name("unix"), kv("target_os", "linux")]))
    );
    assert_eq!(
        "(target.\"unix\" && (target.\"os\" == \"linux\"))",
        &cfg_to_nix_expr(&CfgExpr::All(vec![name("unix"), kv("target_os", "linux")]))
    );
}

/// Escapes a string as a nix string.
///
/// ```
/// use crate2nix::render::escape_nix_string;
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
        if c == '\\' || c == '"' || (c == '$' && peekable_chars.peek() == Some(&'{')) {
            ret.push('\\');
        }
        ret.push(c);
    }
    ret.push('"');
    ret
}
