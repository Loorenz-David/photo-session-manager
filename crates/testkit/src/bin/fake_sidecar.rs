use serde_json::{Value, json};
use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};

fn append(path: &Path, value: &str) {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("record file");
    writeln!(f, "{value}").expect("record line");
}
fn exit(rec: &Path, pid: u32, code: i32) -> ! {
    append(&rec.join("exits"), &format!("{pid} {code}"));
    std::process::exit(code);
}

/// The fake's stdin side: every line is recorded; the last `capture` command's `id` and `cmd_id`
/// are kept for the script's `{capture_id}` / `{cmd_id}` substitution; `quit` exits (P42).
struct Input {
    rx: Receiver<String>,
    rec: PathBuf,
    pid: u32,
    ignore_quit: bool,
    capture_id: String,
    cmd_id: String,
    /// Lines taken and not yet matched by an `await` (a command may arrive before its `await`).
    unmatched: Vec<String>,
}
impl Input {
    fn take(&mut self, line: String) -> String {
        append(&self.rec.join("stdin.log"), &format!("{} {line}", self.pid));
        self.unmatched.push(line.clone());
        let mut words = line.split_whitespace();
        if words.next() == Some("capture") {
            for word in words {
                if let Some(v) = word.strip_prefix("id=") {
                    self.capture_id = v.to_owned();
                } else if let Some(v) = word.strip_prefix("cmd_id=") {
                    self.cmd_id = v.to_owned();
                }
            }
        }
        if line == "quit" && !self.ignore_quit {
            exit(&self.rec, self.pid, 0);
        }
        line
    }
    /// Takes every line already waiting, without blocking.
    fn drain(&mut self) {
        while let Ok(line) = self.rx.try_recv() {
            self.take(line);
        }
    }
    /// Blocks for the next line (`None` once stdin and its reader are gone).
    fn next(&mut self, until: Option<Instant>) -> Option<Result<String, RecvTimeoutError>> {
        let got = match until {
            Some(end) => self
                .rx
                .recv_timeout(end.saturating_duration_since(Instant::now())),
            None => self.rx.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        match got {
            Ok(line) => Some(Ok(self.take(line))),
            Err(RecvTimeoutError::Disconnected) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// The inject channel (P46 L14): once the script has ended, new lines appended to
/// `rec/inject.txt` since this process started run as script directives — a JSON object is
/// printed verbatim, `exit <code>` exits (recorded in `exits`).
struct Inject {
    path: PathBuf,
    offset: u64,
}
impl Inject {
    fn new(rec: &Path) -> Self {
        let path = rec.join("inject.txt");
        let offset = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        Self { path, offset }
    }
    /// The complete lines appended since the last call.
    fn take(&mut self) -> Vec<String> {
        let Ok(bytes) = fs::read(&self.path) else {
            return Vec::new();
        };
        let start = (self.offset as usize).min(bytes.len());
        let Some(end) = bytes[start..].iter().rposition(|b| *b == b'\n') else {
            return Vec::new();
        };
        let end = start + end + 1;
        self.offset = end as u64;
        String::from_utf8_lossy(&bytes[start..end])
            .lines()
            .map(str::to_owned)
            .collect()
    }
}
fn run_injected(line: &str, input: &Input) {
    let line = line.trim();
    if line.starts_with('{') {
        println!("{line}");
        io::stdout().flush().unwrap();
    } else if let Some(code) = line.strip_prefix("exit ") {
        exit(&input.rec, input.pid, code.trim().parse().unwrap_or(0));
    }
}

/// Script mode (P45 §The fake sidecar): one directive per line — a JSON object printed as is
/// after substitution, `sleep <ms>`, `await <cmd>`, `exit <code>`, `stderr <text>`.
fn run_script(text: &str, input: &mut Input) {
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        input.drain();
        if line.starts_with('{') {
            let out = line
                .replace("{capture_id}", &input.capture_id)
                .replace("{cmd_id}", &input.cmd_id);
            println!("{out}");
            io::stdout().flush().unwrap();
        } else if let Some(ms) = line.strip_prefix("sleep ") {
            let end = Instant::now() + Duration::from_millis(ms.trim().parse().unwrap_or(0));
            while Instant::now() < end {
                if input.next(Some(end)).is_none() {
                    thread::sleep(end.saturating_duration_since(Instant::now()));
                }
            }
        } else if let Some(cmd) = line.strip_prefix("await ") {
            let cmd = cmd.trim();
            loop {
                let first_word = |l: &String| l.split_whitespace().next() == Some(cmd);
                if let Some(pos) = input.unmatched.iter().position(first_word) {
                    input.unmatched.drain(..=pos);
                    break;
                }
                if input.next(None).is_none() {
                    exit(&input.rec, input.pid, 0);
                }
            }
        } else if let Some(code) = line.strip_prefix("exit ") {
            exit(&input.rec, input.pid, code.trim().parse().unwrap_or(0));
        } else if let Some(text) = line.strip_prefix("stderr ") {
            eprintln!("{text}");
        }
    }
}

fn main() {
    let exe = env::current_exe().expect("exe");
    let rec = exe.parent().expect("parent").join("rec");
    fs::create_dir_all(&rec).expect("rec");
    let behaviour: Value = fs::read(rec.join("behaviour.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or(json!({}));
    let pid = std::process::id();
    append(&rec.join("pids"), &pid.to_string());
    fs::write(
        rec.join(format!("argv.{pid}.json")),
        serde_json::to_vec(&env::args().collect::<Vec<_>>()).unwrap(),
    )
    .unwrap();
    fs::write(
        rec.join(format!("env.{pid}.json")),
        serde_json::to_vec(&env::vars().collect::<std::collections::HashMap<_, _>>()).unwrap(),
    )
    .unwrap();
    if let Some(stderr) = behaviour.get("stderr").and_then(Value::as_str) {
        eprintln!("{stderr}");
    }
    let script = behaviour
        .get("script")
        .and_then(Value::as_str)
        .map(|path| fs::read_to_string(path).unwrap_or_default());
    if script.is_none() {
        println!(
            "{}",
            json!({"timestamp":"2026-11-06T00:00:00Z","source":"sdk","event":"fake.started","pid":pid})
        );
        io::stdout().flush().unwrap();
    }
    let mut inject = Inject::new(&rec);
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for line in io::stdin().lock().lines() {
            if tx.send(line.unwrap_or_default()).is_err() {
                return;
            }
        }
        let _ = tx.send("quit".into());
    });
    let mut input = Input {
        rx,
        rec: rec.clone(),
        pid,
        ignore_quit: behaviour
            .get("ignore_quit")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        capture_id: String::new(),
        cmd_id: String::new(),
        unmatched: Vec::new(),
    };
    let scripted = script.is_some();
    if let Some(text) = script {
        run_script(&text, &mut input);
    }
    // After the script (or without one): P42 behaviour — record stdin, exit on `quit`, optional
    // timed exit.
    let timer = behaviour
        .get("exit_after_ms")
        .and_then(Value::as_u64)
        .map(|n| Instant::now() + Duration::from_millis(n));
    loop {
        // Script mode tails the inject file every 20 ms between its stdin polls.
        let wake = if scripted {
            let tail = Instant::now() + Duration::from_millis(20);
            Some(timer.map_or(tail, |t| t.min(tail)))
        } else {
            timer
        };
        if scripted {
            for line in inject.take() {
                run_injected(&line, &input);
            }
        }
        match input.next(wake) {
            Some(Ok(_)) => {}
            Some(Err(RecvTimeoutError::Timeout)) if timer.is_some_and(|t| Instant::now() >= t) => {
                exit(&rec, pid, 3)
            }
            Some(Err(RecvTimeoutError::Timeout)) => {}
            Some(Err(RecvTimeoutError::Disconnected)) | None => {
                // stdin closed and the reader is gone: behave as `quit`.
                if !input.ignore_quit {
                    exit(&rec, pid, 0);
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}
