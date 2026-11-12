//! The tool crate's one test program. No test reaches the network: every request goes to `Stub`.

mod u1_probes_and_search;
mod u2_stage_models;

use psm_research_tool::http::{HttpSeam, Wire, WireError, WireReply};
use std::{cell::RefCell, collections::VecDeque, path::Path};

/// The HTTP seam stub: records every request it receives and answers from a queue (an empty
/// queue answers `200 {}`).
#[derive(Default)]
pub struct Stub {
    pub received: RefCell<Vec<Wire>>,
    pub replies: RefCell<VecDeque<WireReply>>,
}

impl Stub {
    pub fn reply(&self, status: u16, body: &str) {
        self.replies.borrow_mut().push_back(WireReply {
            status,
            headers: vec![("content-type".into(), "application/json".into())],
            body: body.as_bytes().to_vec(),
            final_url: String::new(),
            redirects: Vec::new(),
        });
    }
}

impl HttpSeam for Stub {
    fn send(&self, wire: &Wire) -> Result<WireReply, WireError> {
        self.received.borrow_mut().push(wire.clone());
        let mut reply = self.replies.borrow_mut().pop_front().unwrap_or(WireReply {
            status: 200,
            headers: Vec::new(),
            body: b"{}".to_vec(),
            final_url: String::new(),
            redirects: Vec::new(),
        });
        reply.final_url = wire.url.clone();
        Ok(reply)
    }
}

/// Every file under `dir` whose bytes contain `needle`.
pub fn files_containing(dir: &Path, needle: &str) -> Vec<String> {
    let mut hits = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if std::fs::read(&p)
                .map(|b| b.windows(needle.len()).any(|w| w == needle.as_bytes()))
                .unwrap_or(false)
            {
                hits.push(p.display().to_string());
            }
        }
    }
    hits
}

/// The repository root (the tests read the real test file and corrections file).
pub fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
