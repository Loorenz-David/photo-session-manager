use psm_core::{Error, Result};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct OsaRunner {
    data_dir: PathBuf,
}

impl OsaRunner {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
        }
    }

    pub fn run(&self, script_path: &Path, args: &[&str], timeout: Duration) -> Result<String> {
        let start = Instant::now();
        let mut child = Command::new("osascript")
            .arg("-l")
            .arg("JavaScript")
            .arg(script_path)
            .args(args)
            .current_dir(&self.data_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        loop {
            if child.try_wait()?.is_some() {
                let output = child.wait_with_output()?;
                tracing::info!(
                    duration_ms = start.elapsed().as_millis(),
                    command = "osascript",
                    "provider call"
                );
                if !output.status.success() {
                    return Err(Error::Provider(
                        String::from_utf8_lossy(&output.stderr).into_owned(),
                    ));
                }
                return Ok(String::from_utf8_lossy(&output.stdout)
                    .trim_end()
                    .to_string());
            }
            if start.elapsed() >= timeout {
                let _ = child.kill();
                let _ = child.wait();
                tracing::info!(
                    duration_ms = start.elapsed().as_millis(),
                    command = "osascript",
                    "provider timeout"
                );
                return Err(Error::Timeout);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
