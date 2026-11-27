//! The phone wire (`master_plan_live.md` §5 PH4, PH5): error codes, request bodies, the binary
//! frame header. Frozen: LV3's phone app is built against it.
use psm_core::Error;
use serde::{Deserialize, Serialize};

/// PH4: the HTTP status and the code of an error (`{error: {code, message}}`). The codes are the
/// desktop's; an error the phone routes do not expect is `500 internal`.
pub fn error_code(error: &Error) -> (u16, &'static str) {
    match error {
        Error::PairingUnknown => (401, "pairing_unknown"),
        Error::InvalidValue(_) => (400, "invalid_value"),
        Error::NotFound(_) => (404, "not_found"),
        Error::SessionChanged => (409, "session_changed"),
        Error::ShotFilled => (409, "shot_filled"),
        Error::NotInTemplate => (409, "not_in_template"),
        Error::NoTemplate => (409, "no_template"),
        Error::NothingToShift => (409, "nothing_to_shift"),
        Error::CameraNotConnected => (409, "camera_not_connected"),
        Error::FocusUnsupported => (409, "focus_unsupported"),
        Error::SidecarUnavailable => (409, "sidecar_unavailable"),
        Error::CaptureFolderNotSet => (409, "capture_folder_not_set"),
        Error::DocumentNotFront => (409, "document_not_front"),
        Error::CompositionInvalid { .. } => (400, "composition_invalid"),
        Error::FrameUnknown => (409, "frame_unknown"),
        Error::CompositionChanged => (409, "composition_changed"),
        _ => (500, "internal"),
    }
}

/// The `message` of every `500 internal` (LV3·C29: no internal error text reaches a phone).
pub const INTERNAL_MESSAGE: &str = "Something went wrong in Session Manager.";
/// The `message` of every `404 not_found` (LV3·C29: no local path reaches a phone).
pub const NOT_FOUND_MESSAGE: &str = "Not found.";

/// `{error: {code, message}}`. An internal error and a missing photo, container or file answer
/// a fixed text: the store's error string and local paths stay on the Mac.
pub fn error_body(error: &Error) -> serde_json::Value {
    let (status, code) = error_code(error);
    let message = match status {
        500 => INTERNAL_MESSAGE.to_owned(),
        404 => NOT_FOUND_MESSAGE.to_owned(),
        _ => error.to_string(),
    };
    serde_json::json!({"error": {"code": code, "message": message}})
}

/// PH5: the 32-byte little-endian header of a binary frame message: `PSMF`, version 1, flags 0,
/// two reserved bytes, `seq` u64, `gen` u32, width u16, height u16, `captured_at_ms` i64.
pub fn frame_header(
    seq: u64,
    r#gen: u32,
    width: u16,
    height: u16,
    captured_at_ms: i64,
) -> [u8; 32] {
    let mut h = [0u8; 32];
    h[0..4].copy_from_slice(b"PSMF");
    h[4] = 1;
    h[8..16].copy_from_slice(&seq.to_le_bytes());
    h[16..20].copy_from_slice(&r#gen.to_le_bytes());
    h[20..22].copy_from_slice(&width.to_le_bytes());
    h[22..24].copy_from_slice(&height.to_le_bytes());
    h[24..32].copy_from_slice(&captured_at_ms.to_le_bytes());
    h
}

/// `POST /capture`, `/focus/clear`, `/photos/<id>/delete`.
#[derive(Debug, Deserialize)]
pub struct SessionBody {
    pub session_id: i64,
}

/// `POST /focus`: a point of the live frame, 0…1 each.
#[derive(Debug, Deserialize)]
pub struct FocusBody {
    pub session_id: i64,
    pub x: f64,
    pub y: f64,
}

/// `POST /current`: a container, or `null` for Free (Condition).
#[derive(Debug, Deserialize)]
pub struct CurrentBody {
    pub session_id: i64,
    pub container_id: Option<i64>,
}

/// `POST /retake`.
#[derive(Debug, Deserialize)]
pub struct RetakeBody {
    pub session_id: i64,
    pub container_id: i64,
}

/// `POST /photos/<id>/move`.
#[derive(Debug, Deserialize)]
pub struct MoveBody {
    pub session_id: i64,
    pub target: MoveTargetDto,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MoveTargetDto {
    Container {
        container_id: i64,
    },
    MainPosition {
        position: i64,
    },
    ExtraEnd,
    Condition {
        position: i64,
    },
    ConditionEnd,
    /// PH4a: put an extra at place `index` among the extras (1-based).
    ExtraInsert {
        index: i64,
    },
    /// PH4a: put a Condition photo at place `index` among them (1-based).
    ConditionInsert {
        index: i64,
    },
}

/// `POST /photos/<id>/classify` (PH4a).
#[derive(Debug, Deserialize)]
pub struct ClassifyBody {
    pub session_id: i64,
    pub to: ClassifyToDto,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClassifyToDto {
    Main,
    Condition,
}

/// `POST /photos/<id>/composition` (PH4b): `values` in core's serde spellings.
#[derive(Debug, Deserialize)]
pub struct CompositionBody {
    pub session_id: i64,
    pub revision: i64,
    pub values: psm_core::domain::composition::CompositionValues,
}

/// `POST /photos/<id>/composition/reset` (PH4b).
#[derive(Debug, Deserialize)]
pub struct ResetBody {
    pub session_id: i64,
    pub revision: i64,
}

/// `GET /hello`.
#[derive(Debug, Serialize)]
pub struct Hello {
    pub api: u32,
    pub pairing: PairingRef,
    pub mac_name: String,
    pub features: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct PairingRef {
    pub id: i64,
    pub name: String,
}

/// LV-D18: what this server offers (later additions append).
pub const FEATURES: [&str; 5] = ["live", "capture", "focus", "current_shot", "move"];

/// PH4: the web app manifest; the installed app starts with its token.
pub fn manifest(token: &str) -> serde_json::Value {
    serde_json::json!({
        "name": "Shoot",
        "short_name": "Shoot",
        "start_url": format!("/phone/?pair={token}"),
        "scope": "/phone/",
        "display": "standalone",
        "background_color": "#ffffff",
        "theme_color": "#ffffff",
        "icons": [{"src": "/phone/icon.png", "sizes": "512x512", "type": "image/png"}],
    })
}

/// The content type of a static file by its extension.
pub fn content_type(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("html") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json" | "map") => "application/json",
        Some("webmanifest") => "application/manifest+json",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_maps_the_phone_errors_and_hides_the_rest() {
        assert_eq!(error_code(&Error::PairingUnknown), (401, "pairing_unknown"));
        assert_eq!(error_code(&Error::SessionChanged), (409, "session_changed"));
        assert_eq!(error_code(&Error::ShotFilled), (409, "shot_filled"));
        assert_eq!(
            error_code(&Error::CameraNotConnected),
            (409, "camera_not_connected")
        );
        assert_eq!(
            error_code(&Error::InvalidValue("x".into())),
            (400, "invalid_value")
        );
        assert_eq!(error_code(&Error::NotFound("p".into())), (404, "not_found"));
        assert_eq!(error_code(&Error::Timeout), (500, "internal"));
        assert_eq!(
            error_code(&Error::Provider("disk".into())),
            (500, "internal")
        );
    }

    #[test]
    fn frame_header_has_the_ph5_layout() {
        let h = frame_header(0x0102_0304_0506_0708, 7, 1024, 680, -2);
        assert_eq!(&h[0..4], b"PSMF");
        assert_eq!((h[4], h[5], h[6], h[7]), (1, 0, 0, 0));
        assert_eq!(&h[8..16], &0x0102_0304_0506_0708u64.to_le_bytes());
        assert_eq!(&h[16..20], &7u32.to_le_bytes());
        assert_eq!(&h[20..22], &1024u16.to_le_bytes());
        assert_eq!(&h[22..24], &680u16.to_le_bytes());
        assert_eq!(&h[24..32], &(-2i64).to_le_bytes());
    }
}
