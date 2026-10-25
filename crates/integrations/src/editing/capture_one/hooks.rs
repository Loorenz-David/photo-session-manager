use std::{fs, path::Path, process::Command};
use psm_core::{Error, Result};
use psm_infra::paths::AppPaths;

const TEMPLATES: [(&str, &str); 3] = [
    ("capture_done", include_str!("hooks/capture_done.applescript.tmpl")),
    ("processing_done", include_str!("hooks/processing_done.applescript.tmpl")),
    ("variant_adjusted", include_str!("hooks/variant_adjusted.applescript.tmpl")),
];

pub fn applescript_json_escape(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"), '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"), '\r' => out.push_str("\\r"), '\t' => out.push_str("\\t"),
            c if (c as u32) < 32 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

pub fn render_template(tmpl: &str, spool_dir: &Path, port: u16) -> String {
    let path = applescript_json_escape(&spool_dir.to_string_lossy());
    // AppleScript literals also need quotes escaped; the JSON escape covers both.
    tmpl.replace("{{SPOOL_DIR}}", &path).replace("{{PORT}}", &port.to_string())
}

pub fn compile_all(paths: &AppPaths, port: u16) -> Result<()> {
    for (name, tmpl) in TEMPLATES {
        let source = paths.hooks().join(format!("{name}.applescript"));
        let target = paths.hooks().join(format!("{name}.scpt"));
        fs::write(&source, render_template(tmpl, &paths.spool(), port))?;
        let output = Command::new("osacompile").arg("-o").arg(&target).arg(&source).output()?;
        if !output.status.success() { return Err(Error::Provider(format!("osacompile {name}: {}", String::from_utf8_lossy(&output.stderr)))); }
        fs::remove_file(source)?;
    }
    Ok(())
}
