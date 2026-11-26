use psm_core::domain::live::{ExposureReadout, FocusFrame, FocusFrameState, FocusPoint};

pub fn to_grid(point: FocusPoint) -> (u16, u16) {
    let x = (point.x * 639.0).round().clamp(0.0, 639.0) as u16;
    let y = (point.y * 479.0).round().clamp(0.0, 479.0) as u16;
    (x, y)
}
pub fn frame_norm(raw: [i64; 8]) -> Option<FocusFrame> {
    if raw[3] == 0 || raw[5] == 0 {
        return None;
    }
    Some(FocusFrame {
        cx: raw[2] as f64 / raw[3] as f64,
        cy: raw[4] as f64 / raw[5] as f64,
        w: raw[6] as f64 / 640.0,
        h: raw[7] as f64 / 480.0,
        state: match raw[1] {
            2 => FocusFrameState::Focused,
            1 => FocusFrameState::NotFocused,
            _ => FocusFrameState::Unknown,
        },
    })
}
pub fn exposure_readout(shutter: &str, f_number: &str, iso: &str) -> ExposureReadout {
    let f = f_number.strip_suffix(".0").unwrap_or(f_number);
    ExposureReadout {
        shutter: shutter.to_owned(),
        aperture: format!("f/{f}"),
        iso: format!("ISO {iso}"),
    }
}
