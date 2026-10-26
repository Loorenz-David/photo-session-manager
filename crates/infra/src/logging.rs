use crate::paths::AppPaths;
use std::{
    io::{self, Write},
    sync::{OnceLock, RwLock},
};
use tracing_subscriber::fmt::MakeWriter;

static SECRETS: OnceLock<RwLock<Vec<String>>> = OnceLock::new();

/// Registers a value that must never reach a log line (rule 8: the configured client secret,
/// every access token fetched). Every writer made by [`redacting`] replaces it.
pub fn add_secret(secret: &str) {
    if secret.is_empty() {
        return;
    }
    let mut values = SECRETS
        .get_or_init(|| RwLock::new(Vec::new()))
        .write()
        .expect("logging secrets");
    if !values.iter().any(|value| value == secret) {
        values.push(secret.to_owned());
    }
}

/// Core mechanism: `line` with every non-empty secret replaced by `[REDACTED]`.
pub fn redact(line: &str, secrets: &[String]) -> String {
    secrets
        .iter()
        .filter(|secret| !secret.is_empty())
        .fold(line.to_owned(), |text, secret| {
            text.replace(secret, "[REDACTED]")
        })
}

/// Wraps a writer factory so every line written is passed through [`redact`] with the
/// registered secrets (the production log file uses it).
pub fn redacting<M>(inner: M) -> RedactingMakeWriter<M> {
    RedactingMakeWriter(inner)
}

pub struct RedactingMakeWriter<M>(M);
pub struct RedactingWriter<W: Write> {
    inner: W,
    pending: Vec<u8>,
}
impl<'a, M> MakeWriter<'a> for RedactingMakeWriter<M>
where
    M: MakeWriter<'a>,
{
    type Writer = RedactingWriter<M::Writer>;
    fn make_writer(&'a self) -> Self::Writer {
        RedactingWriter {
            inner: self.0.make_writer(),
            pending: Vec::new(),
        }
    }
}
impl<W: Write> RedactingWriter<W> {
    fn drain_lines(&mut self, all: bool) -> io::Result<()> {
        while let Some(end) = self
            .pending
            .iter()
            .position(|byte| *byte == b'\n')
            .or_else(|| all.then_some(self.pending.len()).filter(|n| *n > 0))
        {
            let take = if self.pending.get(end) == Some(&b'\n') {
                end + 1
            } else {
                end
            };
            let line = self.pending.drain(..take).collect::<Vec<_>>();
            let raw = String::from_utf8_lossy(&line);
            let secrets = SECRETS
                .get_or_init(|| RwLock::new(Vec::new()))
                .read()
                .expect("logging secrets");
            self.inner.write_all(redact(&raw, &secrets).as_bytes())?;
        }
        Ok(())
    }
}
impl<W: Write> Write for RedactingWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.pending.extend_from_slice(bytes);
        self.drain_lines(false)?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.drain_lines(true)?;
        self.inner.flush()
    }
}
impl<W: Write> Drop for RedactingWriter<W> {
    fn drop(&mut self) {
        let _ = self.drain_lines(true);
    }
}

pub fn init(paths: &AppPaths) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let file = tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("psm")
        .filename_suffix("log")
        .build(paths.root.join("logs"))
        .ok()?;
    let (writer, guard) = tracing_appender::non_blocking(file);
    let subscriber = tracing_subscriber::fmt()
        .with_writer(redacting(writer))
        .with_ansi(false)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);
    Some(guard)
}

#[cfg(test)]
mod tests {
    use super::redact;
    #[test]
    fn redact_replaces_each_configured_secret() {
        assert_eq!(
            redact(
                "client=SECRETTOKEN123 token=ACCESSTOKEN456",
                &["SECRETTOKEN123".into(), "ACCESSTOKEN456".into()]
            ),
            "client=[REDACTED] token=[REDACTED]"
        );
    }
}
