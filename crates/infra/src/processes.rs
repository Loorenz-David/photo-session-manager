//! The running process names (10E MC-34 R-I30), read with one `ps` listing. Read-only: nothing
//! here can signal or stop a process (M39).
use psm_core::{Error, Result, ports::system::ProcessList};
use std::process::Command;

#[derive(Default)]
pub struct PsProcessList;

/// The process names of a `ps -axo comm=` listing: each line's last path component, trimmed,
/// first occurrence kept, in listing order.
pub fn parse_ps_listing(listing: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for line in listing.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let name = line.rsplit('/').next().unwrap_or(line).trim().to_owned();
        if !name.is_empty() && !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

impl ProcessList for PsProcessList {
    fn running(&self) -> Result<Vec<String>> {
        let output = Command::new("/bin/ps").args(["-axo", "comm="]).output()?;
        if !output.status.success() {
            return Err(Error::Provider(format!(
                "ps: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        Ok(parse_ps_listing(&String::from_utf8_lossy(&output.stdout)))
    }
}
