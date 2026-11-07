//! The camera sidecar's stdio protocol (10E MC-31): stdout carries one JSON event per line,
//! stdin one command per line. Field sets follow the lab's emit sites (P45 plan §`SidecarEvent`):
//! a field written unconditionally at its site is required, every other field is optional.
//! Unknown fields are ignored; an unknown `event` parses to [`SidecarEvent::Unknown`].

use serde_json::{Map, Value};

/// The link a sidecar event names (`transport_manager.cpp` `to_string(Transport)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transport {
    Usb,
    Wifi,
    None,
}

/// The outcome of the connect sequence's store-destination write (R-I19).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreDestination {
    Verified,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SidecarEvent {
    TransportState {
        from: String,
        to: String,
        reason: String,
        transport: Transport,
    },
    ConnectionReady {
        transport: Transport,
        connect_ms: Option<i64>,
        model: Option<String>,
        store_destination: Option<StoreDestination>,
        store_value: Option<String>,
    },
    SessionClosed {
        transport: Transport,
        reason: String,
    },
    SessionLost {
        transport: Transport,
        reason: String,
        busy: String,
    },
    CmdReceived {
        cmd: String,
        cmd_id: Option<String>,
    },
    CmdDone {
        cmd: String,
        cmd_id: Option<String>,
    },
    CmdRejected {
        cmd: String,
        reason: String,
    },
    CmdError {
        cmd: String,
        error: String,
    },
    CaptureRequested {
        capture_id: String,
        transport: Transport,
        af: bool,
    },
    CaptureAf {
        capture_id: String,
        af_result: String,
    },
    CaptureAcknowledged {
        capture_id: String,
        success: bool,
        sdk_error: String,
    },
    CaptureExposed {
        capture_id: Option<String>,
        transport: Transport,
        association: String,
    },
    CaptureExposureMissing {
        capture_id: String,
    },
    CaptureUncertain {
        operation: String,
    },
    CameraBusy {
        reason: String,
        safe_point: bool,
    },
    SwitchDeferred {
        to: String,
        busy: String,
    },
    UsbArrived {
        detail: String,
    },
    UsbRemoved {
        detail: String,
    },
    UsbInitial {
        present: bool,
    },
    UsbClaimedBy {
        owners: String,
        hint: Option<String>,
    },
    UsbFallbackToWifi {
        usb_failures: i64,
        usb_retry: String,
    },
    UsbRetryRequested,
    SdkDownloadCompleted {
        filename: String,
        /// −1 when the sidecar's `file_size` failed.
        size_bytes: i64,
        success: bool,
        transport: Transport,
        r#type: Option<String>,
        provisional_capture_id: Option<String>,
        ms_since_capture_request: Option<i64>,
    },
    SshFingerprint {
        status: String,
        success: Option<bool>,
        fingerprint: Option<String>,
    },
    /// Every non-envelope member of `camera.ftp_state`.
    CameraFtpState {
        fields: Map<String, Value>,
    },
    SdkUnavailable {
        error: String,
    },
    /// `error`, never `message` (`camera_connection.cpp:492–493`, `:511–512`).
    ConnectFailed {
        transport: Transport,
        sdk_error: String,
        error: String,
        timeout: Option<bool>,
    },
    Unknown {
        event: String,
    },
}

/// A parsed event with its envelope timestamp (Mac clock, UTC milliseconds).
#[derive(Clone, Debug, PartialEq)]
pub struct Stamped {
    pub timestamp_ms: i64,
    pub event: SidecarEvent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    /// Not a JSON object, no `event`, or a known event missing a required field.
    Json,
    MissingTimestamp,
    BadTimestamp,
}

/// Envelope members (`json_log.cpp`): never part of an event's own fields.
const ENVELOPE: [&str; 4] = ["timestamp", "experiment", "source", "event"];

/// Parses one stdout line of the sidecar.
pub fn parse_line(line: &str) -> Result<Stamped, ProtocolError> {
    let value: Value = serde_json::from_str(line).map_err(|_| ProtocolError::Json)?;
    let obj = value.as_object().ok_or(ProtocolError::Json)?;
    let timestamp = obj
        .get("timestamp")
        .ok_or(ProtocolError::MissingTimestamp)?;
    let timestamp_ms = timestamp
        .as_str()
        .and_then(iso_to_ms)
        .ok_or(ProtocolError::BadTimestamp)?;
    let name = obj
        .get("event")
        .and_then(Value::as_str)
        .ok_or(ProtocolError::Json)?;
    let f = Fields(obj);
    let event = match name {
        "transport.state" => SidecarEvent::TransportState {
            from: f.req_str("from")?,
            to: f.req_str("to")?,
            reason: f.req_str("reason")?,
            transport: f.transport()?,
        },
        "connection.ready" => SidecarEvent::ConnectionReady {
            transport: f.transport()?,
            connect_ms: f.opt_i64("connect_ms"),
            model: f.opt_str("model"),
            store_destination: match f.opt_str("store_destination").as_deref() {
                None => None,
                Some("verified") => Some(StoreDestination::Verified),
                Some("failed") => Some(StoreDestination::Failed),
                Some(_) => return Err(ProtocolError::Json),
            },
            store_value: f.opt_str("store_value"),
        },
        "session.closed" => SidecarEvent::SessionClosed {
            transport: f.transport()?,
            reason: f.req_str("reason")?,
        },
        "session.lost" => SidecarEvent::SessionLost {
            transport: f.transport()?,
            reason: f.req_str("reason")?,
            busy: f.req_str("busy")?,
        },
        "cmd.received" => SidecarEvent::CmdReceived {
            cmd: f.req_str("cmd")?,
            cmd_id: f.opt_str("cmd_id").filter(|s| !s.is_empty()),
        },
        "cmd.done" => SidecarEvent::CmdDone {
            cmd: f.req_str("cmd")?,
            cmd_id: f.opt_str("cmd_id").filter(|s| !s.is_empty()),
        },
        "cmd.rejected" => SidecarEvent::CmdRejected {
            cmd: f.req_str("cmd")?,
            reason: f.req_str("reason")?,
        },
        "cmd.error" => SidecarEvent::CmdError {
            cmd: f.req_str("cmd")?,
            error: f.req_str("error")?,
        },
        "capture.requested" => SidecarEvent::CaptureRequested {
            capture_id: f.req_str("capture_id")?,
            transport: f.transport()?,
            af: f.req_bool("af")?,
        },
        "capture.af" => SidecarEvent::CaptureAf {
            capture_id: f.req_str("capture_id")?,
            af_result: f.req_str("af_result")?,
        },
        "capture.acknowledged" => SidecarEvent::CaptureAcknowledged {
            capture_id: f.req_str("capture_id")?,
            success: f.req_bool("success")?,
            sdk_error: f.req_str("sdk_error")?,
        },
        "capture.exposed" => SidecarEvent::CaptureExposed {
            capture_id: f.opt_str("capture_id"),
            transport: f.transport()?,
            association: f.req_str("association")?,
        },
        "capture.exposure_missing" => SidecarEvent::CaptureExposureMissing {
            capture_id: f.req_str("capture_id")?,
        },
        "capture.uncertain" => SidecarEvent::CaptureUncertain {
            operation: f.req_str("operation")?,
        },
        "camera.busy" => SidecarEvent::CameraBusy {
            reason: f.req_str("reason")?,
            safe_point: f.req_bool("safe_point")?,
        },
        "switch.deferred" => SidecarEvent::SwitchDeferred {
            to: f.req_str("to")?,
            busy: f.req_str("busy")?,
        },
        "usb.arrived" => SidecarEvent::UsbArrived {
            detail: f.req_str("detail")?,
        },
        "usb.removed" => SidecarEvent::UsbRemoved {
            detail: f.req_str("detail")?,
        },
        "usb.initial" => SidecarEvent::UsbInitial {
            present: f.req_bool("present")?,
        },
        "usb.claimed_by" => SidecarEvent::UsbClaimedBy {
            owners: f.req_str("owners")?,
            hint: f.opt_str("hint"),
        },
        "usb.fallback_to_wifi" => SidecarEvent::UsbFallbackToWifi {
            usb_failures: f.req_i64("usb_failures")?,
            usb_retry: f.req_str("usb_retry")?,
        },
        "usb.retry_requested" => SidecarEvent::UsbRetryRequested,
        "sdk.download.completed" => SidecarEvent::SdkDownloadCompleted {
            filename: f.req_str("filename")?,
            size_bytes: f.req_i64("size_bytes")?,
            success: f.req_bool("success")?,
            transport: f.transport()?,
            r#type: f.opt_str("type"),
            provisional_capture_id: f.opt_str("provisional_capture_id"),
            ms_since_capture_request: f.opt_i64("ms_since_capture_request"),
        },
        "ssh.fingerprint" => SidecarEvent::SshFingerprint {
            status: f.req_str("status")?,
            success: f.opt_bool("success"),
            fingerprint: f.opt_str("fingerprint"),
        },
        "camera.ftp_state" => SidecarEvent::CameraFtpState {
            fields: obj
                .iter()
                .filter(|(k, _)| !ENVELOPE.contains(&k.as_str()))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        },
        "sdk.unavailable" => SidecarEvent::SdkUnavailable {
            error: f.req_str("error")?,
        },
        "connect.failed" => SidecarEvent::ConnectFailed {
            transport: f.transport()?,
            sdk_error: f.req_str("sdk_error")?,
            error: f.req_str("error")?,
            timeout: f.opt_bool("timeout"),
        },
        other => SidecarEvent::Unknown {
            event: other.to_owned(),
        },
    };
    Ok(Stamped {
        timestamp_ms,
        event,
    })
}

struct Fields<'a>(&'a Map<String, Value>);
impl Fields<'_> {
    fn req_str(&self, key: &str) -> Result<String, ProtocolError> {
        self.opt_str(key).ok_or(ProtocolError::Json)
    }
    fn opt_str(&self, key: &str) -> Option<String> {
        self.0.get(key).and_then(Value::as_str).map(str::to_owned)
    }
    fn req_bool(&self, key: &str) -> Result<bool, ProtocolError> {
        self.opt_bool(key).ok_or(ProtocolError::Json)
    }
    fn opt_bool(&self, key: &str) -> Option<bool> {
        self.0.get(key).and_then(Value::as_bool)
    }
    fn req_i64(&self, key: &str) -> Result<i64, ProtocolError> {
        self.opt_i64(key).ok_or(ProtocolError::Json)
    }
    fn opt_i64(&self, key: &str) -> Option<i64> {
        self.0.get(key).and_then(Value::as_i64)
    }
    fn transport(&self) -> Result<Transport, ProtocolError> {
        match self.0.get("transport").and_then(Value::as_str) {
            Some("usb") => Ok(Transport::Usb),
            Some("wifi") => Ok(Transport::Wifi),
            Some("none") => Ok(Transport::None),
            _ => Err(ProtocolError::Json),
        }
    }
}

/// `YYYY-MM-DDTHH:MM:SS[.f{1,9}]Z` (UTC) → Unix milliseconds; the fraction is truncated to ms.
/// Any other form → `None`.
pub fn iso_to_ms(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() < 20 || *b.last()? != b'Z' {
        return None;
    }
    let digits = |r: std::ops::Range<usize>| -> Option<i64> {
        let part = b.get(r)?;
        if part.is_empty() || !part.iter().all(u8::is_ascii_digit) {
            return None;
        }
        Some(part.iter().fold(0i64, |n, d| n * 10 + i64::from(d - b'0')))
    };
    if b[4] != b'-' || b[7] != b'-' || b[10] != b'T' || b[13] != b':' || b[16] != b':' {
        return None;
    }
    let (year, month, day) = (digits(0..4)?, digits(5..7)?, digits(8..10)?);
    let (hour, minute, second) = (digits(11..13)?, digits(14..16)?, digits(17..19)?);
    let rest = &b[19..b.len() - 1];
    let millis = match rest {
        [] => 0,
        [b'.', frac @ ..]
            if (1..=9).contains(&frac.len()) && frac.iter().all(u8::is_ascii_digit) =>
        {
            let mut ms = 0i64;
            for i in 0..3 {
                ms = ms * 10 + frac.get(i).map_or(0, |d| i64::from(d - b'0'));
            }
            ms
        }
        _ => return None,
    };
    if !(1..=12).contains(&month)
        || !(1..=days_in_month(year, month)).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }
    let days = days_from_civil(year, month, day);
    Some((((days * 24 + hour) * 60 + minute) * 60 + second) * 1000 + millis)
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 of a proleptic Gregorian date (H. Hinnant's algorithm).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// RFC 3986 percent-encoding: the unreserved set (`A–Z a–z 0–9 - . _ ~`) is kept, every other
/// byte of the UTF-8 encoding becomes `%XX` (upper-case hex). The sidecar reverses it.
pub fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// A command for the sidecar's `auto` mode (lab handoff §4.3; R-I21, R-I23, R-I29).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Only ever the first stdin line (R-I21); never logged.
    Credentials {
        user: String,
        password: String,
    },
    Capture {
        id: String,
        af: bool,
        cmd_id: String,
    },
    Props {
        cmd_id: String,
    },
    FtpStatus {
        cmd_id: String,
    },
    Status {
        cmd_id: String,
    },
    UsbRetry {
        cmd_id: String,
    },
    /// Close and reopen the Wi-Fi session (R-I29; the sidecar refuses it on USB).
    Reconnect {
        cmd_id: String,
    },
    Quit,
}

impl Command {
    /// The stdin line (without the newline). An empty `cmd_id` is left out.
    pub fn encode(&self) -> String {
        let with_id = |head: String, cmd_id: &str| {
            if cmd_id.is_empty() {
                head
            } else {
                format!("{head} cmd_id={cmd_id}")
            }
        };
        match self {
            Command::Credentials { user, password } => format!(
                "credentials user={} password={}",
                percent_encode(user),
                percent_encode(password)
            ),
            Command::Capture { id, af, cmd_id } => {
                with_id(format!("capture id={id} af={}", u8::from(*af)), cmd_id)
            }
            Command::Props { cmd_id } => with_id("props".into(), cmd_id),
            Command::FtpStatus { cmd_id } => with_id("ftp-status".into(), cmd_id),
            Command::Status { cmd_id } => with_id("status".into(), cmd_id),
            Command::UsbRetry { cmd_id } => with_id("usb-retry".into(), cmd_id),
            Command::Reconnect { cmd_id } => with_id("reconnect".into(), cmd_id),
            Command::Quit => "quit".into(),
        }
    }
    /// The correlation id the command carries (`None` for `credentials` and `quit`).
    pub fn cmd_id(&self) -> Option<&str> {
        match self {
            Command::Capture { cmd_id, .. }
            | Command::Props { cmd_id }
            | Command::FtpStatus { cmd_id }
            | Command::Status { cmd_id }
            | Command::UsbRetry { cmd_id }
            | Command::Reconnect { cmd_id } => Some(cmd_id),
            Command::Credentials { .. } | Command::Quit => None,
        }
    }
    /// The same command with `cmd_id` replaced (no-op for commands without one).
    pub fn with_cmd_id(mut self, id: &str) -> Self {
        match &mut self {
            Command::Capture { cmd_id, .. }
            | Command::Props { cmd_id }
            | Command::FtpStatus { cmd_id }
            | Command::Status { cmd_id }
            | Command::UsbRetry { cmd_id }
            | Command::Reconnect { cmd_id } => *cmd_id = id.to_owned(),
            Command::Credentials { .. } | Command::Quit => {}
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAB: &str = include_str!("fixtures/lab_auto_switching.jsonl");
    const EMIT_SITES: &str = include_str!("fixtures/emit_sites.jsonl");

    fn s(v: &str) -> String {
        v.to_owned()
    }
    fn events(text: &str) -> Vec<SidecarEvent> {
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| parse_line(l).unwrap_or_else(|e| panic!("{e:?}: {l}")).event)
            .collect()
    }

    /// C3(a)'s expected sequence (handoff §4.2, the 13 real lines).
    fn lab_sequence() -> Vec<SidecarEvent> {
        use SidecarEvent::*;
        let id = s("53d7…");
        vec![
            TransportState {
                from: s("DISCONNECTED"),
                to: s("DISCOVERING"),
                reason: s("looking for camera over wifi"),
                transport: Transport::None,
            },
            ConnectionReady {
                transport: Transport::Wifi,
                connect_ms: Some(914),
                model: None,
                store_destination: None,
                store_value: None,
            },
            CmdReceived {
                cmd: s("capture"),
                cmd_id: Some(s("4db6be91")),
            },
            CameraBusy {
                reason: s("exposure"),
                safe_point: false,
            },
            CaptureRequested {
                capture_id: id.clone(),
                transport: Transport::Wifi,
                af: true,
            },
            CaptureAf {
                capture_id: id.clone(),
                af_result: s("focused"),
            },
            CaptureAcknowledged {
                capture_id: id.clone(),
                success: true,
                sdk_error: s("0x00000000"),
            },
            CaptureExposed {
                capture_id: Some(id),
                transport: Transport::Wifi,
                association: s("latest_release_provisional"),
            },
            CmdDone {
                cmd: s("capture"),
                cmd_id: Some(s("4db6be91")),
            },
            UsbArrived {
                detail: s("pid=0x0E0C ILCE-7RM5"),
            },
            SessionClosed {
                transport: Transport::Wifi,
                reason: s("switching wifi -> usb"),
            },
            SessionLost {
                transport: Transport::Usb,
                reason: s("usb device removed"),
                busy: s("none"),
            },
            SwitchDeferred {
                to: s("usb"),
                busy: s("pixel_shift"),
            },
        ]
    }

    #[test]
    fn mc31_lab_lines_parse() {
        assert_eq!(events(LAB), lab_sequence());
        let second = LAB.lines().nth(1).unwrap();
        assert_eq!(parse_line(second).unwrap().timestamp_ms, 1_793_888_519_760);
    }

    #[test]
    fn mc31_emit_site_lines_parse() {
        use SidecarEvent::*;
        let hint = s(
            "another process holds the camera's USB interface; the SDK cannot open it. ptpcamerad claims cameras for apps using Image Capture (e.g. Capture One, Photos). Quit that app, then replug.",
        );
        let expected = vec![
            UsbClaimedBy {
                owners: s("pid 512, ptpcamerad"),
                hint: Some(hint),
            },
            UsbFallbackToWifi {
                usb_failures: 3,
                usb_retry: s("on next USB re-plug"),
            },
            SdkDownloadCompleted {
                filename: s("/Users/x/Session/Capture/.incoming/sdk-r1/DSC00007.ARW"),
                size_bytes: 51_234_567,
                success: true,
                transport: Transport::Usb,
                r#type: Some(s("0x00000001")),
                provisional_capture_id: Some(s("c1")),
                ms_since_capture_request: Some(1960),
            },
            SdkDownloadCompleted {
                filename: s("/Users/x/Session/Capture/.incoming/sdk-r1/X.ARW"),
                size_bytes: -1,
                success: false,
                transport: Transport::Usb,
                r#type: None,
                provisional_capture_id: None,
                ms_since_capture_request: None,
            },
            Unknown {
                event: s("sdk.download.hashed"),
            },
            SshFingerprint {
                status: s("unknown"),
                success: Some(false),
                fingerprint: None,
            },
            SshFingerprint {
                status: s("MISMATCH"),
                success: Some(false),
                fingerprint: None,
            },
            CaptureUncertain {
                operation: s("bracketing"),
            },
            CaptureExposureMissing {
                capture_id: s("c0"),
            },
            CmdRejected {
                cmd: s("capture"),
                reason: s("not connected (DISCOVERING)"),
            },
            CmdError {
                cmd: s("x"),
                error: s("unknown command"),
            },
            SdkUnavailable {
                error: s("camera-sidecar was built without the Sony SDK"),
            },
            ConnectFailed {
                transport: Transport::Wifi,
                sdk_error: s("0x00008208"),
                error: s("connection failed: connection error"),
                timeout: Some(false),
            },
            ConnectFailed {
                transport: Transport::Usb,
                sdk_error: s("0x00008208"),
                error: s("connection error"),
                timeout: None,
            },
            UsbInitial { present: true },
            ConnectionReady {
                transport: Transport::Usb,
                connect_ms: Some(1200),
                model: Some(s("ILCE-7RM5")),
                store_destination: Some(StoreDestination::Failed),
                store_value: Some(s("0x00000002")),
            },
        ];
        assert_eq!(events(EMIT_SITES), expected);
    }

    #[test]
    fn mc31_unknown_ignored_malformed_rejected() {
        assert_eq!(
            parse_line(r#"{"timestamp":"2026-11-06T00:00:00.000Z","source":"sdk","event":"camera.newthing","x":1}"#)
                .unwrap()
                .event,
            SidecarEvent::Unknown {
                event: s("camera.newthing")
            }
        );
        assert_eq!(
            parse_line(r#"{"timestamp":"2026-11-06T00:00:00.000Z","source":"sdk","event":"connection.ready","transport":"wifi","connect_ms":914,"total_ms":4675,"foo":"bar"}"#)
                .unwrap()
                .event,
            SidecarEvent::ConnectionReady {
                transport: Transport::Wifi,
                connect_ms: Some(914),
                model: None,
                store_destination: None,
                store_value: None,
            }
        );
        assert_eq!(parse_line("not json"), Err(ProtocolError::Json));
        assert_eq!(
            parse_line(
                r#"{"source":"sdk","event":"connection.ready","transport":"wifi","connect_ms":1}"#
            ),
            Err(ProtocolError::MissingTimestamp)
        );
        let no_fraction = parse_line(
            r#"{"timestamp":"2026-11-06T00:00:00Z","source":"sdk","event":"usb.initial","present":false}"#,
        )
        .unwrap();
        assert_eq!(
            no_fraction.event,
            SidecarEvent::UsbInitial { present: false }
        );
        assert_eq!(no_fraction.timestamp_ms, 1_793_923_200_000);
        assert_eq!(
            parse_line(
                r#"{"timestamp":"2026-11-06 00:00:00","source":"sdk","event":"usb.initial","present":false}"#
            ),
            Err(ProtocolError::BadTimestamp)
        );
        assert_eq!(
            parse_line(
                r#"{"timestamp":"2026-11-06T00:00:00.000Z","source":"sdk","event":"connection.ready"}"#
            ),
            Err(ProtocolError::Json)
        );
    }

    #[test]
    fn mc31_iso_to_ms() {
        assert_eq!(iso_to_ms("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(
            iso_to_ms("2026-11-05T14:21:59.760Z"),
            Some(1_793_888_519_760)
        );
        assert_eq!(iso_to_ms("2026-11-05T14:21:59.7Z"), Some(1_793_888_519_700));
        assert_eq!(
            iso_to_ms("2026-11-05T14:21:59.760999999Z"),
            Some(1_793_888_519_760)
        );
        assert_eq!(iso_to_ms("2026-11-05T14:21:59.Z"), None);
        assert_eq!(iso_to_ms("2026-11-05T14:21:59.7600000000Z"), None);
        assert_eq!(iso_to_ms("2026-11-05T14:21:59"), None);
        assert_eq!(iso_to_ms("2026-02-30T00:00:00Z"), None);
    }

    #[test]
    fn mc31_credentials_encoding() {
        assert_eq!(
            Command::Credentials {
                user: s("u name"),
                password: s("p/w=&é")
            }
            .encode(),
            "credentials user=u%20name password=p%2Fw%3D%26%C3%A9"
        );
        assert_eq!(percent_encode("Az09-._~"), "Az09-._~");
    }

    #[test]
    fn mc31_capture_encoding() {
        assert_eq!(
            Command::Capture {
                id: s("11111111-2222-4333-8444-555555555555"),
                af: true,
                cmd_id: s("c1")
            }
            .encode(),
            "capture id=11111111-2222-4333-8444-555555555555 af=1 cmd_id=c1"
        );
    }

    #[test]
    fn mc31_other_commands_encoding() {
        assert_eq!(
            Command::UsbRetry { cmd_id: s("c2") }.encode(),
            "usb-retry cmd_id=c2"
        );
        assert_eq!(
            Command::FtpStatus { cmd_id: s("c3") }.encode(),
            "ftp-status cmd_id=c3"
        );
        assert_eq!(
            Command::Props { cmd_id: s("c4") }.encode(),
            "props cmd_id=c4"
        );
        assert_eq!(
            Command::Status { cmd_id: s("c5") }.encode(),
            "status cmd_id=c5"
        );
        assert_eq!(Command::Quit.encode(), "quit");
    }
}
