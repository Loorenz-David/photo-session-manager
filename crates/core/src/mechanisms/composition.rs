//! MC-27 geometry shared with the Compose editor through `composition_cases.json`.
//!
//! `Frame { w, h }` is Capture One's crop-ignored export after its own orientation.
//! Quarter turns and horizontal flip run first; `turned_frame` swaps dimensions for
//! odd turns. Straighten then rotates clockwise on screen about the centre, and the
//! fractional rect is measured in that turned frame with the origin at the top left.
//! A split forces a 2:1 rect; `orientation = turned` inverts a named shape ratio.
//!
//! Validation reports the first of: turns_range, straighten_range, fraction_range,
//! rect_outside, rect_too_small, off_shape or split_ratio, then rect_outside again
//! if inverse-rotated corners leave the source by more than 1e-6 of its long side.
//! The short rect side must be at least 5% of the turned frame's short side, and a
//! locked ratio may differ by at most one source pixel.
//!
//! Pixel coordinates round half away from zero. A split's width rounds down to an
//! even value and becomes two adjacent, equally wide square halves. Fingerprints
//! compare Capture One geometry to 1e-6 on reals, exact flip and amount, source
//! orientation, and pixel dimensions only when both observations are masters.
use crate::domain::composition::{
    CompositionKind, CompositionValues, Fingerprint, Frame, FrameSource, Orientation, PixelRect,
    Rect, ReviewReason, Shape,
};

pub const MIN_RECT_FRACTION: f64 = 0.05;

pub fn turned_frame(frame: Frame, quarter_turns: u8) -> Frame {
    if quarter_turns % 2 == 1 {
        Frame {
            w: frame.h,
            h: frame.w,
        }
    } else {
        frame
    }
}
pub fn orientation_of(frame: Frame) -> &'static str {
    if frame.w > frame.h {
        "landscape"
    } else if frame.w < frame.h {
        "portrait"
    } else {
        "square"
    }
}
pub fn shape_ratio(
    kind: CompositionKind,
    shape: Shape,
    orientation: Orientation,
    frame: Frame,
) -> Option<f64> {
    if kind == CompositionKind::Split {
        return Some(2.0);
    }
    let r = match shape {
        Shape::Original => return Some(frame.w as f64 / frame.h as f64),
        Shape::Square => 1.0,
        Shape::FourFive => 4.0 / 5.0,
        Shape::ThreeTwo => 3.0 / 2.0,
        Shape::TwoThree => 2.0 / 3.0,
        Shape::SixteenNine => 16.0 / 9.0,
        Shape::Free => return None,
    };
    Some(if orientation == Orientation::Turned {
        1.0 / r
    } else {
        r
    })
}
pub fn default_rect(
    kind: CompositionKind,
    shape: Shape,
    orientation: Orientation,
    turned: Frame,
) -> Rect {
    let Some(r) = shape_ratio(kind, shape, orientation, turned) else {
        return Rect {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
        };
    };
    let frame_ratio = turned.w as f64 / turned.h as f64;
    if r >= frame_ratio {
        let h = frame_ratio / r;
        Rect {
            x: 0.0,
            y: (1.0 - h) / 2.0,
            w: 1.0,
            h,
        }
    } else {
        let w = r / frame_ratio;
        Rect {
            x: (1.0 - w) / 2.0,
            y: 0.0,
            w,
            h: 1.0,
        }
    }
}
pub fn inside_after_straighten(rect: Rect, turned: Frame, straighten: f64) -> bool {
    let w = turned.w as f64;
    let h = turned.h as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let (sin, cos) = straighten.to_radians().sin_cos();
    let epsilon = 1e-6 * w.max(h);
    for px in [rect.x * w, (rect.x + rect.w) * w] {
        for py in [rect.y * h, (rect.y + rect.h) * h] {
            let dx = px - cx;
            let dy = py - cy;
            let sx = cx + cos * dx + sin * dy;
            let sy = cy - sin * dx + cos * dy;
            if sx < -epsilon || sx > w + epsilon || sy < -epsilon || sy > h + epsilon {
                return false;
            }
        }
    }
    true
}
pub fn validate(values: &CompositionValues, frame: Frame) -> std::result::Result<(), &'static str> {
    if values.quarter_turns > 3 {
        return Err("turns_range");
    }
    if !values.straighten.is_finite() || values.straighten.abs() > 45.0 {
        return Err("straighten_range");
    }
    let r = values.rect;
    if [r.x, r.y, r.w, r.h]
        .iter()
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        || r.w <= 0.0
        || r.h <= 0.0
    {
        return Err("fraction_range");
    }
    if r.x + r.w > 1.0 + 1e-9 || r.y + r.h > 1.0 + 1e-9 {
        return Err("rect_outside");
    }
    let t = turned_frame(frame, values.quarter_turns);
    if (r.w * t.w as f64).min(r.h * t.h as f64) < MIN_RECT_FRACTION * t.w.min(t.h) as f64 {
        return Err("rect_too_small");
    }
    if let Some(ratio) = shape_ratio(values.kind, values.shape, values.orientation, t) {
        if values.kind == CompositionKind::Normal
            && (r.w * t.w as f64 - ratio * r.h * t.h as f64).abs() > 1.0
        {
            return Err("off_shape");
        }
        if values.kind == CompositionKind::Split
            && (r.w * t.w as f64 - 2.0 * r.h * t.h as f64).abs() > 1.0
        {
            return Err("split_ratio");
        }
    }
    if values.straighten != 0.0 && !inside_after_straighten(r, t, values.straighten) {
        return Err("rect_outside");
    }
    Ok(())
}
pub fn pixel_rect(rect: Rect, turned: Frame) -> PixelRect {
    PixelRect {
        left: (rect.x * turned.w as f64).round() as i64,
        top: (rect.y * turned.h as f64).round() as i64,
        width: (rect.w * turned.w as f64).round() as i64,
        height: (rect.h * turned.h as f64).round() as i64,
    }
}
pub fn split_halves(pixel: PixelRect) -> (PixelRect, PixelRect) {
    let half = (pixel.width / 2).max(0);
    let left = PixelRect {
        width: half,
        height: half,
        ..pixel
    };
    let right = PixelRect {
        left: pixel.left + half,
        width: half,
        height: half,
        ..pixel
    };
    (left, right)
}
pub fn geometry_equal(
    a: &crate::domain::composition::EditorGeometry,
    b: &crate::domain::composition::EditorGeometry,
) -> bool {
    (a.rotation - b.rotation).abs() <= 1e-6
        && (a.keystone_vertical - b.keystone_vertical).abs() <= 1e-6
        && (a.keystone_horizontal - b.keystone_horizontal).abs() <= 1e-6
        && a.flip == b.flip
        && a.keystone_amount == b.keystone_amount
}
pub fn fingerprint_compatible(stored: &Fingerprint, observed: &Fingerprint) -> bool {
    geometry_equal(&stored.geometry, &observed.geometry)
        && orientation_of(stored.frame) == orientation_of(observed.frame)
        && !(stored.source == FrameSource::Master
            && observed.source == FrameSource::Master
            && stored.frame != observed.frame)
}
pub fn review_reason(stored: &Fingerprint, observed: &Fingerprint) -> ReviewReason {
    if geometry_equal(&stored.geometry, &observed.geometry) {
        ReviewReason::Frame
    } else {
        ReviewReason::Geometry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::composition::{CompositionKind, Orientation, Shape};
    use serde::de::DeserializeOwned;
    use serde_json::Value;
    fn cases() -> Vec<Value> {
        serde_json::from_str::<Value>(include_str!("composition_cases.json")).unwrap()["cases"]
            .as_array()
            .unwrap()
            .clone()
    }
    fn parse<T: DeserializeOwned>(v: &Value) -> T {
        serde_json::from_value(v.clone()).unwrap()
    }
    fn close(a: f64, b: f64) {
        assert!((a - b).abs() <= 1e-9, "{a} != {b}");
    }
    fn rect_eq(a: Rect, b: Rect) {
        close(a.x, b.x);
        close(a.y, b.y);
        close(a.w, b.w);
        close(a.h, b.h);
    }
    #[test]
    fn c1a_default_rect() {
        for c in cases().into_iter().filter(|c| c["op"] == "default_rect") {
            let i = &c["input"];
            let frame: Frame = parse(&i["frame"]);
            let got = default_rect(
                parse::<CompositionKind>(&i["kind"]),
                parse::<Shape>(&i["shape"]),
                parse::<Orientation>(&i["orientation"]),
                turned_frame(frame, i["turns"].as_u64().unwrap() as u8),
            );
            rect_eq(got, parse(&c["expect"]));
        }
    }
    #[test]
    fn c1b_pixel_rect_and_halves() {
        for c in cases()
            .into_iter()
            .filter(|c| c["op"] == "pixel_rect" || c["op"] == "split_halves")
        {
            let i = &c["input"];
            if c["op"] == "pixel_rect" {
                assert_eq!(
                    pixel_rect(parse(&i["rect"]), parse(&i["frame"])),
                    parse(&c["expect"]),
                    "{}",
                    c["id"]
                );
            } else {
                let (left, right) = split_halves(parse(&i["pixel"]));
                assert_eq!(left, parse(&c["expect"]["left"]));
                assert_eq!(right, parse(&c["expect"]["right"]));
            }
        }
    }
    #[test]
    fn c1c_validate() {
        for c in cases().into_iter().filter(|c| c["op"] == "validate") {
            let i = &c["input"];
            let result = validate(&parse(&i["values"]), parse(&i["frame"]));
            let actual = result.err().unwrap_or("ok");
            assert_eq!(
                actual,
                c["expect"]["result"].as_str().unwrap(),
                "{}",
                c["id"]
            );
        }
    }
    #[test]
    fn c1d_fingerprint_and_inside() {
        for c in cases()
            .into_iter()
            .filter(|c| c["op"] == "fingerprint" || c["op"] == "inside")
        {
            let i = &c["input"];
            if c["op"] == "fingerprint" {
                let a: Fingerprint = parse(&i["stored"]);
                let b: Fingerprint = parse(&i["observed"]);
                assert_eq!(
                    fingerprint_compatible(&a, &b),
                    c["expect"]["compatible"].as_bool().unwrap(),
                    "{}",
                    c["id"]
                );
                if !fingerprint_compatible(&a, &b) {
                    assert_eq!(
                        review_reason(&a, &b).as_str(),
                        c["expect"]["reason"].as_str().unwrap()
                    );
                }
            } else {
                assert_eq!(
                    inside_after_straighten(
                        parse(&i["rect"]),
                        parse(&i["frame"]),
                        i["straighten"].as_f64().unwrap()
                    ),
                    c["expect"]["inside"].as_bool().unwrap()
                );
            }
        }
    }
}
