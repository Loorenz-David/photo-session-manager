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
    fn part(s: &str) -> &str {
        if s.is_empty() || s == "special" {
            "–"
        } else {
            s
        }
    }
    let f = part(f_number);
    let aperture = if let Some(number) = f.strip_prefix('F') {
        format!("f/{}", number.strip_suffix(".0").unwrap_or(number))
    } else {
        f.to_owned()
    };
    let iso = part(iso);
    ExposureReadout {
        shutter: part(shutter).to_owned(),
        aperture,
        iso: if iso == "AUTO" {
            "ISO AUTO".into()
        } else {
            iso.to_owned()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::exposure_readout;
    #[test]
    fn lv5_c12c_exposure_display_mapping() {
        let cases = [
            (("1/80", "F8.0", "ISO 2000"), ("1/80", "f/8", "ISO 2000")),
            (("2.5\"", "F5.6", "AUTO"), ("2.5\"", "f/5.6", "ISO AUTO")),
            (("BULB", "F11.0", "ISO 100"), ("BULB", "f/11", "ISO 100")),
            (("special", "special", ""), ("–", "–", "–")),
        ];
        for (input, expected) in cases {
            let actual = exposure_readout(input.0, input.1, input.2);
            assert_eq!(
                (
                    actual.shutter.as_str(),
                    actual.aperture.as_str(),
                    actual.iso.as_str()
                ),
                expected
            );
        }
    }
}
