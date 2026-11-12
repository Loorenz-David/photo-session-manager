use super::{RESTART_BACKOFF_S, RESTART_STEADY_S};
use std::{
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct SidecarSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub first_line: String,
}
pub enum SupervisorEvent {
    Started { pid: u32 },
    Exited { code: Option<i32> },
    Line(String),
    Stderr(String),
}
enum ReaderEvent {
    Line(String),
    Stderr(String),
}

pub struct Supervisor {
    pub spec: SidecarSpec,
    child: Option<Child>,
    attempts: usize,
    started_at: Option<Instant>,
    next_spawn_at: Option<Instant>,
    tx: Sender<ReaderEvent>,
    rx: Receiver<ReaderEvent>,
    stopped: bool,
}
impl Supervisor {
    pub fn new(spec: SidecarSpec) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            spec,
            child: None,
            attempts: 0,
            started_at: None,
            next_spawn_at: None,
            tx,
            rx,
            stopped: false,
        }
    }
    pub fn spawn(&mut self, now: Instant) -> std::io::Result<SupervisorEvent> {
        let mut cmd = Command::new(&self.spec.program);
        cmd.args(&self.spec.args)
            .envs(self.spec.env.iter().map(|(k, v)| (k, v)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn()?;
        let pid = child.id();
        if let Some(mut input) = child.stdin.take() {
            input.write_all(self.spec.first_line.as_bytes())?;
            input.write_all(b"\n")?;
            child.stdin = Some(input);
        }
        if let Some(out) = child.stdout.take() {
            let tx = self.tx.clone();
            thread::spawn(move || {
                for line in BufReader::new(out).lines().map_while(Result::ok) {
                    let _ = tx.send(ReaderEvent::Line(line));
                }
            });
        }
        if let Some(err) = child.stderr.take() {
            let tx = self.tx.clone();
            thread::spawn(move || {
                for line in BufReader::new(err).lines().map_while(Result::ok) {
                    let _ = tx.send(ReaderEvent::Stderr(line));
                }
            });
        }
        self.started_at = Some(now);
        self.next_spawn_at = None;
        self.child = Some(child);
        Ok(SupervisorEvent::Started { pid })
    }
    pub fn tick(&mut self, now: Instant) -> Vec<SupervisorEvent> {
        let mut events = Vec::new();
        if self.stopped {
            return events;
        }
        while let Ok(e) = self.rx.try_recv() {
            events.push(match e {
                ReaderEvent::Line(line) => SupervisorEvent::Line(line),
                ReaderEvent::Stderr(line) => SupervisorEvent::Stderr(line),
            });
        }
        if let Some(child) = self.child.as_mut()
            && let Ok(Some(status)) = child.try_wait()
        {
            if self
                .started_at
                .is_some_and(|at| now.saturating_duration_since(at) >= Duration::from_secs(60))
            {
                self.attempts = 0;
            }
            self.attempts += 1;
            let wait = if self.attempts == 1 {
                0
            } else if self.attempts - 2 < RESTART_BACKOFF_S.len() {
                RESTART_BACKOFF_S[self.attempts - 2]
            } else {
                RESTART_STEADY_S
            };
            self.next_spawn_at = Some(now + Duration::from_secs(wait));
            self.child = None;
            events.push(SupervisorEvent::Exited {
                code: status.code(),
            });
        }
        if self.child.is_none() && self.next_spawn_at.is_some_and(|at| now >= at) {
            match self.spawn(now) {
                Ok(e) => events.push(e),
                Err(e) => {
                    events.push(SupervisorEvent::Stderr(format!(
                        "sidecar spawn failed: {e}"
                    )));
                    self.next_spawn_at = Some(now + Duration::from_secs(RESTART_STEADY_S));
                }
            }
        }
        events
    }
    /// Writes one line (plus `\n`) to the running sidecar's stdin. Stdin stays open for the
    /// sidecar's lifetime: its end of file means `quit`.
    pub fn send_line(&mut self, line: &str) -> std::io::Result<()> {
        let input = self
            .child
            .as_mut()
            .and_then(|child| child.stdin.as_mut())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "camera sidecar not running",
                )
            })?;
        input.write_all(line.as_bytes())?;
        input.write_all(b"\n")?;
        input.flush()
    }
    pub fn consecutive_exits(&self) -> usize {
        self.attempts
    }
    /// Sends `quit` and waits up to `grace` (production: `SidecarTimings::default().quit_grace`)
    /// for the sidecar to exit, then kills it.
    pub fn quit_and_wait(&mut self, grace: Duration) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        let Some(mut child) = self.child.take() else {
            return;
        };
        if let Some(mut input) = child.stdin.take() {
            let _ = input.write_all(b"quit\n");
            let _ = input.flush();
        }
        let end = Instant::now() + grace;
        loop {
            match child.try_wait() {
                Ok(Some(_)) => return,
                Err(_) => break,
                Ok(None) => {}
            }
            if Instant::now() >= end {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let _ = child.kill();
        let _ = child.wait();
    }
}
