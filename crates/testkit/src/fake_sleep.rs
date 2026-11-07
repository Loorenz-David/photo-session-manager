//! `FakeSleepInhibitor` (P48): records the idle-sleep assertion instead of creating one.
use psm_core::{Result, ports::system::SleepInhibitor};
use std::sync::Mutex;

#[derive(Default)]
pub struct FakeSleepInhibitor {
    state: Mutex<(Option<String>, usize)>,
}
impl FakeSleepInhibitor {
    /// The reason of the assertion held now.
    pub fn held(&self) -> Option<String> {
        self.state.lock().unwrap().0.clone()
    }
    /// How many times `hold` was called.
    pub fn holds(&self) -> usize {
        self.state.lock().unwrap().1
    }
}
impl SleepInhibitor for FakeSleepInhibitor {
    fn hold(&self, reason: &str) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.0 = Some(reason.to_owned());
        state.1 += 1;
        Ok(())
    }
    fn release(&self) {
        self.state.lock().unwrap().0 = None;
    }
}
