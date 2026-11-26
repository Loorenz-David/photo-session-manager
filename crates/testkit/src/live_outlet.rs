use std::sync::{Arc, Mutex};
use psm_core::{domain::live::{LiveEvent, LiveFrame}, services::live_view::{LatestSlot, LiveOutlet}};

type EventTrace = Arc<Mutex<Vec<String>>>;

#[derive(Default)]
pub struct RecordingOutlet {
    pub offers: Mutex<Vec<(Arc<LiveFrame>,u32)>>,
    pub events: Mutex<Vec<LiveEvent>>,
    pub withdraws: Mutex<usize>,
    pub slot: LatestSlot<(Arc<LiveFrame>,u32)>,
    pub trace: Mutex<Option<(String,EventTrace)>>,
}
impl RecordingOutlet {
    pub fn take(&self) -> Option<(Arc<LiveFrame>,u32)> { self.slot.take() }
    pub fn offered(&self) -> Vec<(Arc<LiveFrame>,u32)> { self.offers.lock().unwrap().clone() }
    pub fn events(&self) -> Vec<LiveEvent> { self.events.lock().unwrap().clone() }
    pub fn withdrawals(&self) -> usize { *self.withdraws.lock().unwrap() }
}
impl LiveOutlet for RecordingOutlet {
    fn offer(&self, frame: Arc<LiveFrame>, generation: u32) {
        self.slot.put((frame.clone(),generation));
        self.offers.lock().unwrap().push((frame,generation));
    }
    fn withdraw(&self) { self.slot.clear(); *self.withdraws.lock().unwrap()+=1; }
    fn event(&self,event:&LiveEvent) {
        self.events.lock().unwrap().push(event.clone());
        if let Some((label,trace))=&*self.trace.lock().unwrap() {trace.lock().unwrap().push(label.clone());}
    }
}
