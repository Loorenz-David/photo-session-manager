use serde_json::{Value, json};
use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, BufRead, Write},
    path::Path,
    sync::mpsc,
    thread,
    time::Duration,
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
    println!(
        "{}",
        json!({"timestamp":"2026-11-06T00:00:00Z","source":"sdk","event":"fake.started","pid":pid})
    );
    io::stdout().flush().unwrap();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for line in io::stdin().lock().lines() {
            if tx.send(line.unwrap_or_default()).is_err() {
                return;
            }
        }
        let _ = tx.send("quit".into());
    });
    let timer = behaviour
        .get("exit_after_ms")
        .and_then(Value::as_u64)
        .map(|n| std::time::Instant::now() + Duration::from_millis(n));
    loop {
        let line = if let Some(end) = timer {
            match rx.recv_timeout(end.saturating_duration_since(std::time::Instant::now())) {
                Ok(line) => line,
                Err(mpsc::RecvTimeoutError::Timeout) => exit(&rec, pid, 3),
                Err(_) => "quit".into(),
            }
        } else {
            rx.recv().unwrap_or_else(|_| "quit".into())
        };
        append(&rec.join("stdin.log"), &format!("{pid} {line}"));
        if line == "quit"
            && !behaviour
                .get("ignore_quit")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        {
            exit(&rec, pid, 0);
        }
    }
}
