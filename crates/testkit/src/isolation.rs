//! Per-test process isolation inside the one suite program (P70).
//!
//! The behaviour tests share one test binary, so its tests run as threads of one process. A test
//! that installs a **process-wide** log subscriber (and reads every line the app writes, from any
//! thread) would otherwise see the lines of every other test running at the same time. Such a
//! test wraps its body in [`in_own_process`]: the body runs in a child process of the same test
//! binary that runs only that one test, exactly as it did when each phase file was its own program.

use std::process::Command;

/// Set (to the test's full name) in the child process that runs one isolated test.
const CHILD: &str = "PSM_TESTKIT_ISOLATED_TEST";

/// Runs `body` in a child process of this test binary that runs only the test `name` within
/// `module` (pass `module_path!()`); the parent asserts the child passed and shows its output if not.
pub fn in_own_process(module: &str, name: &str, body: impl FnOnce()) {
    // `module_path!()` starts with the test crate's name; libtest names omit it.
    let module = module.split_once("::").map_or("", |(_, rest)| rest);
    let test = if module.is_empty() {
        name.to_owned()
    } else {
        format!("{module}::{name}")
    };
    if std::env::var(CHILD).is_ok_and(|value| value == test) {
        body();
        return;
    }
    let output = Command::new(std::env::current_exe().expect("test binary path"))
        .args([test.as_str(), "--exact", "--test-threads=1", "--nocapture"])
        .env(CHILD, &test)
        .output()
        .expect("start the isolated test process");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("1 passed"),
        "isolated test {test} failed ({}):\n{stdout}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}
