/// Item types are the same despite casing or whitespace at their ends.
pub fn type_key(item_type: &str) -> String {
    item_type.trim().to_lowercase()
}

use crate::domain::composition::{
    CompositionKind, CompositionValues, Frame, Orientation, Rect, Shape,
};
use crate::domain::template::{Container, Presets};
use crate::mechanisms::composition::{default_rect, turned_frame};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expected<'a> {
    Main(&'a Container),
    Condition,
}

pub fn expected(index: usize, containers: &[Container]) -> Expected<'_> {
    containers
        .iter()
        .find(|c| c.shoot_no == index as i64)
        .map_or(Expected::Condition, Expected::Main)
}

pub fn zoomed(rect: Rect, zoom: u8) -> Rect {
    let factor = 1.0 - f64::from(zoom) / 100.0;
    let w = rect.w * factor;
    let h = rect.h * factor;
    Rect {
        x: rect.x + (rect.w - w) / 2.0,
        y: rect.y + (rect.h - h) / 2.0,
        w,
        h,
    }
}

/// PH6 (`master_plan_live.md` §5): the width/height ratio the phone shows for a shot: split → 2.0;
/// else the presets' shape (or `default_shape` when the container has none): `1:1` 1.0, `4:5` 0.8,
/// `3:2` 1.5, `2:3` 0.6667, `16:9` 1.7778; `original` and `free` → `None` (the phone uses the live
/// frame's aspect).
pub fn shot_ratio(presets: &Presets, default_shape: Shape) -> Option<f64> {
    if presets.split {
        return Some(2.0);
    }
    match presets.shape.unwrap_or(default_shape) {
        Shape::Square => Some(1.0),
        Shape::FourFive => Some(0.8),
        Shape::ThreeTwo => Some(1.5),
        Shape::TwoThree => Some(2.0 / 3.0),
        Shape::SixteenNine => Some(16.0 / 9.0),
        Shape::Original | Shape::Free => None,
    }
}

pub fn preset_values(
    presets: &Presets,
    current: &CompositionValues,
    frame: Frame,
) -> CompositionValues {
    let kind = if presets.split {
        CompositionKind::Split
    } else {
        CompositionKind::Normal
    };
    let shape = presets.shape.unwrap_or(current.shape);
    let orientation = Orientation::Named;
    let rect = zoomed(
        default_rect(
            kind,
            shape,
            orientation,
            turned_frame(frame, current.quarter_turns),
        ),
        presets.zoom,
    );
    CompositionValues {
        kind,
        shape,
        orientation,
        rect,
        ..current.clone()
    }
}

#[cfg(test)]
mod tests {
    fn shaped(shape: Option<Shape>) -> Presets {
        Presets {
            split: false,
            shape,
            zoom: 0,
        }
    }
    fn ratio_near(value: Option<f64>, want: f64) -> bool {
        value.is_some_and(|v| (v - want).abs() < 1e-4)
    }

    /// LV2·C12(b): the PH6 ratio table.
    #[test]
    fn lv2_c12b_shot_ratio_follows_the_ph6_table() {
        let split = Presets {
            split: true,
            shape: Some(Shape::ThreeTwo),
            zoom: 20,
        };
        assert_eq!(shot_ratio(&split, Shape::Square), Some(2.0));
        assert!(ratio_near(
            shot_ratio(&shaped(Some(Shape::Square)), Shape::Original),
            1.0
        ));
        assert!(ratio_near(
            shot_ratio(&shaped(Some(Shape::FourFive)), Shape::Square),
            0.8
        ));
        assert!(ratio_near(
            shot_ratio(&shaped(Some(Shape::ThreeTwo)), Shape::Square),
            1.5
        ));
        assert!(ratio_near(
            shot_ratio(&shaped(Some(Shape::TwoThree)), Shape::Square),
            0.6667
        ));
        assert!(ratio_near(
            shot_ratio(&shaped(Some(Shape::SixteenNine)), Shape::Square),
            1.7778
        ));
        assert_eq!(
            shot_ratio(&shaped(Some(Shape::Original)), Shape::Square),
            None
        );
        assert_eq!(shot_ratio(&shaped(Some(Shape::Free)), Shape::Square), None);
        assert!(ratio_near(shot_ratio(&shaped(None), Shape::Square), 1.0));
    }

    use super::*;
    use crate::domain::composition::{Flip, Shape};
    use crate::mechanisms::composition::validate;

    #[test]
    fn t1_c4a_type_key() {
        assert_eq!(type_key("Dining chair"), "dining chair");
        assert_eq!(type_key("  STOL "), "stol");
        assert_eq!(type_key("Ö-stol"), "ö-stol");
    }

    fn chair() -> Vec<Container> {
        (1..=3)
            .map(|i| Container {
                id: i,
                final_no: i,
                shoot_no: [2, 1, 3][(i - 1) as usize],
                example_file: None,
                example_path: None,
                presets: Presets::default(),
                name: None,
                description: None,
            })
            .collect()
    }
    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }
    fn rect_is(a: Rect, b: Rect) {
        close(a.x, b.x);
        close(a.y, b.y);
        close(a.w, b.w);
        close(a.h, b.h);
    }
    #[test]
    fn t2_c2a_shoot_to_final() {
        let containers = chair();
        assert!(
            matches!(expected(1,&containers), Expected::Main(c) if c.id == 2 && c.final_no == 2)
        );
        assert!(
            matches!(expected(2,&containers), Expected::Main(c) if c.id == 1 && c.final_no == 1)
        );
        assert!(
            matches!(expected(3,&containers), Expected::Main(c) if c.id == 3 && c.final_no == 3)
        );
        assert!(matches!(expected(4, &containers), Expected::Condition));
    }
    #[test]
    fn t2_c2b_zoom_centred() {
        rect_is(
            zoomed(
                Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 1.0,
                    h: 1.0,
                },
                20,
            ),
            Rect {
                x: 0.1,
                y: 0.1,
                w: 0.8,
                h: 0.8,
            },
        );
        rect_is(
            zoomed(
                Rect {
                    x: 0.0,
                    y: 0.25,
                    w: 1.0,
                    h: 0.5,
                },
                50,
            ),
            Rect {
                x: 0.25,
                y: 0.375,
                w: 0.5,
                h: 0.25,
            },
        );
        let r = Rect {
            x: 0.1,
            y: 0.2,
            w: 0.3,
            h: 0.4,
        };
        rect_is(zoomed(r, 0), r);
    }
    #[test]
    fn t2_c2c_preset_values() {
        let frame = Frame { w: 6000, h: 4000 };
        let original = CompositionValues {
            kind: CompositionKind::Normal,
            shape: Shape::Original,
            orientation: Orientation::Named,
            quarter_turns: 0,
            flip: Flip::None,
            straighten: 0.0,
            rect: Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            },
        };
        let split = preset_values(
            &Presets {
                split: true,
                shape: None,
                zoom: 20,
            },
            &original,
            frame,
        );
        assert_eq!(split.kind, CompositionKind::Split);
        rect_is(
            split.rect,
            Rect {
                x: 0.1,
                y: 0.2,
                w: 0.8,
                h: 0.6,
            },
        );
        assert!(validate(&split, frame).is_ok());
        let square = preset_values(
            &Presets {
                split: false,
                shape: Some(Shape::Square),
                zoom: 0,
            },
            &original,
            frame,
        );
        assert_eq!(square.kind, CompositionKind::Normal);
        assert_eq!(square.shape, Shape::Square);
        rect_is(
            square.rect,
            Rect {
                x: 1.0 / 6.0,
                y: 0.0,
                w: 2.0 / 3.0,
                h: 1.0,
            },
        );
    }
}
