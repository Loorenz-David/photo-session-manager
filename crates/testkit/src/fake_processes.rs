//! `FakeProcessList` (P48): the process names the camera ownership guard reads; empty by default.
//! It has no kill API — nothing in the app can stop a process (M39).
use psm_core::{Result, ports::system::ProcessList};
use std::sync::Mutex;

#[derive(Default)]
pub struct FakeProcessList {
    names: Mutex<Vec<String>>,
}
impl FakeProcessList {
    pub fn set(&self, names: Vec<&str>) {
        *self.names.lock().unwrap() = names.into_iter().map(str::to_owned).collect();
    }
    pub fn names(&self) -> Vec<String> {
        self.names.lock().unwrap().clone()
    }
}
impl ProcessList for FakeProcessList {
    fn running(&self) -> Result<Vec<String>> {
        Ok(self.names())
    }
}
