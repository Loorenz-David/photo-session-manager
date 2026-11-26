//! Binary sidecar frames. The listener and the JSON control client use separate channels.
use psm_core::{
    domain::{
        camera::Link,
        live::{CameraFrame, SourceStop},
    },
    services::live_view::LiveViewHub,
};
use std::{
    collections::VecDeque,
    fs,
    io::{self, Read},
    os::unix::{
        fs::PermissionsExt,
        net::{UnixListener, UnixStream},
    },
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

const HEADER: usize = 24;
const MAX_JPEG: usize = 4_194_304;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameRecord {
    pub epoch: u32,
    pub frame_no: u32,
    pub width: u16,
    pub height: u16,
    pub jpeg: Vec<u8>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    Header,
    Length,
}
pub fn parse_record(bytes: &[u8]) -> Result<FrameRecord, FrameError> {
    if bytes.len() < HEADER
        || &bytes[..4] != b"PSLV"
        || bytes[4] != 1
        || bytes[5] != 1
        || bytes[6..8] != [0, 0]
    {
        return Err(FrameError::Header);
    }
    let u32at = |p: usize| u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap());
    let u16at = |p: usize| u16::from_le_bytes(bytes[p..p + 2].try_into().unwrap());
    let len = u32at(20) as usize;
    if !(1..=MAX_JPEG).contains(&len) || bytes.len() != HEADER + len {
        return Err(FrameError::Length);
    }
    Ok(FrameRecord {
        epoch: u32at(8),
        frame_no: u32at(12),
        width: u16at(16),
        height: u16at(18),
        jpeg: bytes[HEADER..].to_vec(),
    })
}
struct State {
    active: Option<(u32, Link)>,
    connection: u64,
    started: bool,
    bad: bool,
}
pub struct FrameListener {
    path: PathBuf,
    state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    hub: Arc<LiveViewHub>,
}
impl FrameListener {
    pub fn bind(
        run_id: &str,
        hub: Arc<LiveViewHub>,
        helper_log: Arc<Mutex<VecDeque<String>>>,
        previous_connections: u64,
    ) -> io::Result<Self> {
        let short = run_id.chars().take(12).collect::<String>();
        let name = format!("psm-lv-{short}.sock");
        let mut path = std::env::temp_dir().join(&name);
        if path.to_string_lossy().len() > 104 {
            path = PathBuf::from("/tmp").join(name);
        }
        let _ = fs::remove_file(&path);
        let listener = UnixListener::bind(&path)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        listener.set_nonblocking(true)?;
        let state = Arc::new(Mutex::new(State {
            active: None,
            connection: previous_connections,
            started: false,
            bad: false,
        }));
        let stop = Arc::new(AtomicBool::new(false));
        let thread_state = state.clone();
        let thread_stop = stop.clone();
        let thread_hub = hub.clone();
        let thread = thread::Builder::new()
            .name("camera-frames".into())
            .spawn(move || {
                while !thread_stop.load(Ordering::SeqCst) {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            stream
                                .set_read_timeout(Some(Duration::from_millis(30)))
                                .ok();
                            {
                                let mut s = thread_state.lock().unwrap();
                                if s.bad {
                                    continue;
                                }
                                s.connection += 1;
                                s.started = false;
                                if let Some((epoch, link)) = s.active.clone() {
                                    thread_hub.source_started(
                                        link,
                                        (s.connection << 32) | u64::from(epoch),
                                    );
                                    s.started = true;
                                }
                            }
                            Self::read_stream(
                                &mut stream,
                                &thread_state,
                                &thread_stop,
                                &thread_hub,
                                &helper_log,
                            );
                        }
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(5))
                        }
                        Err(_) => break,
                    }
                }
            })?;
        Ok(Self {
            path,
            state,
            stop,
            thread: Some(thread),
            hub,
        })
    }
    fn read_full(stream: &mut UnixStream, buf: &mut [u8], stop: &AtomicBool) -> io::Result<()> {
        let mut offset = 0;
        while offset < buf.len() {
            if stop.load(Ordering::SeqCst) {
                return Err(io::ErrorKind::Interrupted.into());
            }
            match stream.read(&mut buf[offset..]) {
                Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(n) => offset += n,
                Err(e)
                    if e.kind() == io::ErrorKind::WouldBlock
                        || e.kind() == io::ErrorKind::TimedOut
                        || e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
    fn read_stream(
        stream: &mut UnixStream,
        state: &Mutex<State>,
        stop: &AtomicBool,
        hub: &LiveViewHub,
        log: &Mutex<VecDeque<String>>,
    ) {
        while !stop.load(Ordering::SeqCst) {
            let (epoch, link, source) = loop {
                if stop.load(Ordering::SeqCst) {
                    return;
                }
                let s = state.lock().unwrap();
                if s.bad {
                    return;
                }
                if let Some((epoch, link)) = s.active.clone() {
                    break (epoch, link, (s.connection << 32) | u64::from(epoch));
                }
                drop(s);
                thread::sleep(Duration::from_millis(5));
            };
            let mut header = [0u8; HEADER];
            if Self::read_full(stream, &mut header, stop).is_err() {
                break;
            }
            let len = u32::from_le_bytes(header[20..24].try_into().unwrap()) as usize;
            if len == 0
                || len > MAX_JPEG
                || &header[..4] != b"PSLV"
                || header[4] != 1
                || header[5] != 1
                || header[6..8] != [0, 0]
            {
                Self::bad(state, hub, log);
                break;
            }
            let mut bytes = Vec::with_capacity(HEADER + len);
            bytes.extend_from_slice(&header);
            bytes.resize(HEADER + len, 0);
            if Self::read_full(stream, &mut bytes[HEADER..], stop).is_err() {
                break;
            }
            match parse_record(&bytes) {
                Ok(record) if record.epoch == epoch => {
                    hub.publish(CameraFrame {
                        source,
                        frame_no: record.frame_no,
                        width: record.width,
                        height: record.height,
                        jpeg: record.jpeg,
                        link,
                    });
                }
                Ok(_) => {}
                Err(_) => {
                    Self::bad(state, hub, log);
                    break;
                }
            }
        }
        if !stop.load(Ordering::SeqCst) {
            let mut s = state.lock().unwrap();
            if !s.bad {
                s.active = None;
                s.started = false;
                drop(s);
                hub.source_stopped(SourceStop::SidecarExit);
            }
        }
    }
    fn bad(state: &Mutex<State>, hub: &LiveViewHub, log: &Mutex<VecDeque<String>>) {
        let mut s = state.lock().unwrap();
        s.bad = true;
        s.active = None;
        drop(s);
        hub.source_stopped(SourceStop::BadRecord);
        log.lock()
            .unwrap()
            .push_back("live.stop reason=bad_record".into());
        tracing::warn!("live.stop reason=bad_record");
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn connection_count(&self) -> u64 {
        self.state.lock().unwrap().connection
    }
    pub fn start_source(&self, epoch: u32, link: Link) {
        let mut s = self.state.lock().unwrap();
        s.active = Some((epoch, link.clone()));
        if s.connection > 0 && !s.started && !s.bad {
            self.hub
                .source_started(link, (s.connection << 32) | u64::from(epoch));
            s.started = true;
        }
    }
    pub fn stop_source(&self, reason: SourceStop) {
        let mut s = self.state.lock().unwrap();
        s.active = None;
        s.started = false;
        drop(s);
        self.hub.source_stopped(reason);
    }
    pub fn stop(&mut self) {
        if self.stop.swap(true, Ordering::SeqCst) {
            return;
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        let _ = fs::remove_file(&self.path);
    }
}
impl Drop for FrameListener {
    fn drop(&mut self) {
        self.stop();
    }
}
