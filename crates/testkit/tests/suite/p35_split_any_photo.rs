use psm_core::domain::composition::{
    CompositionKind, CompositionValues, Flip, Orientation, Rect, Shape,
};
use psm_core::domain::photo::Classification;
use psm_testkit::TestApp;
use psm_testkit::fixtures::synthetic_jpeg::SyntheticJpeg;

#[test]
fn c4c_unreviewed_photo_accepts_and_stores_a_split() {
    let a = TestApp::with_fake_document().unwrap();
    let session = a
        .items()
        .create_local("Split", "SPLIT", "object", None)
        .unwrap()
        .1
        .session
        .id;
    let photo = a.add_photo(session, "unreviewed.ARW").unwrap();
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
    a.app.previews.request_initial(&photo.raw_path).unwrap();
    let issued = a
        .fake
        .state
        .lock()
        .unwrap()
        .issued
        .last()
        .unwrap()
        .0
        .clone();
    a.complete_render_with(&issued, &SyntheticJpeg::rgb(2560, 1707, [80, 80, 80]))
        .unwrap();
    assert_eq!(
        a.store().photo(photo.id).unwrap().classification,
        Classification::Unreviewed
    );

    let rect = Rect {
        x: 0.028503408013501264,
        y: 0.1464466094067262,
        w: 0.9429931839729975,
        h: std::f64::consts::FRAC_1_SQRT_2,
    };
    let stored = a
        .app
        .compositions
        .set(
            photo.id,
            CompositionValues {
                kind: CompositionKind::Split,
                shape: Shape::Square,
                orientation: Orientation::Named,
                quarter_turns: 0,
                flip: Flip::None,
                straighten: 0.0,
                rect,
            },
        )
        .unwrap();
    assert_eq!(stored.kind, CompositionKind::Split);
    assert_eq!(stored.revision, 1);
    assert_eq!(
        stored.state,
        psm_core::domain::composition::CompositionState::Ok
    );
    let read = a.composition(photo.id).unwrap();
    assert_eq!(read.kind, CompositionKind::Split);
    assert!((read.rect.unwrap().x - rect.x).abs() < 1e-12);
    assert!((read.rect.unwrap().y - rect.y).abs() < 1e-12);
    assert!((read.rect.unwrap().w - rect.w).abs() < 1e-12);
    assert!((read.rect.unwrap().h - rect.h).abs() < 1e-12);
}
