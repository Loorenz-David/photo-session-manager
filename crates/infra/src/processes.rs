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

/// A20: the lowest process id among the lines of a `ps -axo pid=,comm=` listing whose name (last
/// path component) is `name`.
pub fn parse_ps_pid(listing: &str, name: &str) -> Option<u32> {
    listing
        .lines()
        .filter_map(|line| {
            let (pid, comm) = line.trim().split_once(char::is_whitespace)?;
            let comm = comm.trim();
            (comm.rsplit('/').next().unwrap_or(comm).trim() == name)
                .then(|| pid.trim().parse::<u32>().ok())
                .flatten()
        })
        .min()
}

impl PsProcessList {
    /// A20: the process id of the running program named `name` (one `ps` listing; never a signal).
    pub fn pid(&self, name: &str) -> Result<Option<u32>> {
        let output = Command::new("/bin/ps")
            .args(["-axo", "pid=,comm="])
            .output()?;
        if !output.status.success() {
            return Err(Error::Provider(format!(
                "ps: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        Ok(parse_ps_pid(&String::from_utf8_lossy(&output.stdout), name))
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a20_pid_of_capture_one_from_a_listing() {
        let listing = "  1 /sbin/launchd\n54852 /Applications/Capture One.app/Contents/MacOS/Capture One\n54864 /Applications/Capture One.app/Contents/Frameworks/X.framework/CODiagnosticsService\n20592 /Applications/Session Manager.app/Contents/MacOS/psm-desktop\n";
        assert_eq!(parse_ps_pid(listing, "Capture One"), Some(54852));
        assert_eq!(parse_ps_pid(listing, "psm-desktop"), Some(20592));
        assert_eq!(parse_ps_pid(listing, "Capture"), None);
        assert_eq!(parse_ps_pid("", "Capture One"), None);
    }
}
