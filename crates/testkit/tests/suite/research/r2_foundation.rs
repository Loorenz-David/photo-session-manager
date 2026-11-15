use image::GenericImageView;
use std::sync::Arc;

use psm_core::domain::composition::{
    CompositionKind, CompositionValues, Flip, Orientation, Rect, Shape,
};
use psm_core::ports::images::Imaging;
use psm_core::{
    Error,
    domain::{
        attribute::{Provenance, Value},
        session::{ListFilter, PublishStep, RowLabel, SessionStatus},
    },
    mechanisms::status::{StatusInput, derive_status, row_label},
    ports::{commerce::CommerceAdapter, store::StoreTx},
};
use psm_infra::imaging::ImageOpsImpl;
use psm_infra::{paths::AppPaths, sqlite::store::SqliteStore};
use psm_testkit::TestApp;
use rusqlite::{Connection, params};
use rusqlite_migration::Migrations;

#[test]
fn a07_c7a_schema_step_preserves_rows_and_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    let before = SqliteStore::migrations().len();
    let mut conn = Connection::open(paths.db()).unwrap();
    Migrations::new(SqliteStore::migrations())
        .to_latest(&mut conn)
        .unwrap();
    conn.execute("INSERT INTO items(id,sku,folder_key,folder_key_ci,item_type,title,origin,created_at,updated_at) VALUES (1,'R2','R2','r2','object','Test','local',0,0)", []).unwrap();
    conn.execute("INSERT INTO attribute_values(item_id,key,value,valid,provenance,updated_at) VALUES (1,'x','\"Oak\"',1,'human',1)", []).unwrap();
    drop(conn);
    for _ in 0..2 {
        let store = SqliteStore::open(&paths).unwrap();
        drop(store);
        let conn = Connection::open(paths.db()).unwrap();
        let row: (String, Option<String>) = conn
            .query_row(
                "SELECT value,ai_prior FROM attribute_values WHERE item_id=1 AND key='x'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(row, ("\"Oak\"".into(), None));
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM research_feedback", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(publish_items)")
            .unwrap()
            .query_map([], |r| r.get(1))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        for name in [
            "frozen_attribute_origin",
            "pending_conflicts",
            "resolved_attributes",
            "released_at",
        ] {
            assert!(columns.iter().any(|c| c == name), "{name}");
        }
    }
    assert_eq!(SqliteStore::migrations().len(), before);
}

#[test]
fn a09_c5a_held_status_and_unresolved_label() {
    let input = StatusInput {
        latest_step: Some(PublishStep::Held),
        failed_step: None,
        published_at: None,
        dirty: false,
        main_count: 1,
        unreviewed_count: 0,
        non_deleted_missing_preview_count: 0,
        uploaded: 0,
        total: 1,
        unresolved_conflicts: 1,
    };
    assert_eq!(derive_status(input), SessionStatus::Publishing);
    assert_eq!(row_label(input), RowLabel::WaitingForYou(1));
}

#[test]
fn a09_c5b_held_session_is_publishing_in_list() {
    let a = TestApp::new().unwrap();
    let (_, created) = a
        .items()
        .create_local("R2-HELD", "Held", "object", None)
        .unwrap();
    let session = created.session.id;
    a.insert_publish_item(session, "held", 100, None).unwrap();
    let conn = Connection::open(a.app.paths.db()).unwrap();
    conn.execute(
        "UPDATE publish_items SET pending_conflicts=?1,resolved_attributes=?2 WHERE session_id=?3",
        params![r#"{"a":{},"b":{}}"#, r#"{"a":{}}"#, session],
    )
    .unwrap();
    let row = a
        .sessions()
        .list(ListFilter::All, None)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .find(|r| r.id == session)
        .unwrap();
    assert_eq!(row.status, SessionStatus::Publishing);
    assert_eq!(row.label, RowLabel::WaitingForYou(1));
}

#[test]
fn f_c2a_research_transaction_commits_and_rolls_back_both_views() {
    let a = TestApp::new().unwrap();
    let (item, _) = a
        .items()
        .create_local("R2-TX", "Transaction", "object", None)
        .unwrap();
    let value = Value {
        item_id: item.id,
        key: "x".into(),
        value: "\"first\"".into(),
        valid: true,
        provenance: Provenance::Human,
        ai_confidence: None,
        ai_status: None,
        ai_reason: None,
        ai_sources: None,
        ai_prior: None,
        updated_at: 1,
    };
    a.app
        .store
        .research_transaction(|tx| {
            tx.conn()
                .execute("CREATE TABLE scratch (value TEXT)", [])
                .map_err(|e| Error::InvalidData(e.to_string()))?;
            tx.conn()
                .execute("INSERT INTO scratch(value) VALUES ('first')", [])
                .map_err(|e| Error::InvalidData(e.to_string()))?;
            tx.put_attribute_value(&value)?;
            Ok(())
        })
        .unwrap();
    let rejected: psm_core::Result<()> = a.app.store.research_transaction(|tx| {
        tx.conn()
            .execute("INSERT INTO scratch(value) VALUES ('second')", [])
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        tx.put_attribute_value(&Value {
            value: "\"second\"".into(),
            ..value.clone()
        })?;
        Err(Error::InvalidValue("abort".into()))
    });
    assert!(rejected.is_err());
    let conn = Connection::open(a.app.paths.db()).unwrap();
    let scratch: Vec<String> = conn
        .prepare("SELECT value FROM scratch")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(scratch, vec!["first"]);
    let stored = a.store().transaction(&mut |tx| {
        assert_eq!(tx.attribute_values(item.id)?[0].value, "\"first\"");
        Ok(())
    });
    stored.unwrap();
}

#[test]
fn f_c3a_public_listing_default_and_fake_values() {
    let a = TestApp::new().unwrap();
    // R8·U3 implements `public_listing` for Shopify (rows A17·C4 in `r8_shop_facts.rs`), so the
    // Shopify adapter no longer shows the empty default; the fake's values are checked here.
    let fake = a.commerce.unwrap_or_else(|| {
        Arc::new(psm_testkit::fake_commerce::FakeCommerce::new(
            a.clock.clone(),
        ))
    });
    fake.set_public_listing(
        vec!["shop.example".into()],
        Some("https://shop.example/item/1".into()),
        "/item/1".into(),
    );
    let listed = fake.public_listing("remote-id").unwrap();
    assert_eq!(listed.hosts, vec!["shop.example"]);
    assert_eq!(
        listed.item_url.as_deref(),
        Some("https://shop.example/item/1")
    );
    assert_eq!(listed.item_path_tail, "/item/1");
}

fn original() -> CompositionValues {
    CompositionValues {
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
    }
}

fn marked_preview(w: u32, h: u32) -> Vec<u8> {
    let mut img = image::RgbImage::from_pixel(w, h, image::Rgb([220, 220, 220]));
    let size = w.min(h) / 10;
    for y in 0..size {
        for x in 0..size {
            img.put_pixel(x, y, image::Rgb([240, 20, 20]));
            img.put_pixel(w - 1 - x, y, image::Rgb([20, 240, 20]));
            img.put_pixel(x, h - 1 - y, image::Rgb([20, 20, 240]));
            img.put_pixel(w - 1 - x, h - 1 - y, image::Rgb([240, 200, 20]));
        }
    }
    let mut bytes = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 90)
        .encode_image(&img)
        .unwrap();
    bytes
}

fn has_colour(img: &image::RgbImage, test: impl Fn([u8; 3]) -> bool) -> bool {
    img.pixels().any(|p| test(p.0))
}

#[test]
fn a10_c1a_full_frame_reduced_with_corners() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("adjusted.jpg");
    let dest = dir.path().join("research.jpg");
    std::fs::write(&source, marked_preview(3000, 2000)).unwrap();
    let info = ImageOpsImpl
        .render_research(&source, &original(), 1024, &dest)
        .unwrap();
    assert_eq!((info.width, info.height), (1024, 683));
    let img = image::open(&dest).unwrap().into_rgb8();
    assert_eq!(img.dimensions(), (1024, 683));
    let near = |x, y| img.get_pixel(x, y).0;
    let red = near(20, 20);
    let green = near(1000, 20);
    let blue = near(20, 660);
    let yellow = near(1000, 660);
    assert!(red[0] > 180 && red[1] < 70 && red[2] < 70);
    assert!(green[1] > 180 && green[0] < 70 && green[2] < 70);
    assert!(blue[2] > 180 && blue[0] < 70 && blue[1] < 70);
    assert!(yellow[0] > 180 && yellow[1] > 130 && yellow[2] < 70);
}

#[test]
fn a10_c1b_turn_and_cut_right_half_of_source() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("adjusted.jpg");
    let dest = dir.path().join("research.jpg");
    std::fs::write(&source, marked_preview(3000, 2000)).unwrap();
    let mut values = original();
    values.quarter_turns = 1;
    // MC-27 stores the rect in the turned frame; its bottom half is the source's right half.
    values.rect = Rect {
        x: 0.0,
        y: 0.5,
        w: 1.0,
        h: 0.5,
    };
    ImageOpsImpl
        .render_research(&source, &values, 1024, &dest)
        .unwrap();
    let img = image::open(&dest).unwrap().into_rgb8();
    assert_eq!(img.dimensions(), (1024, 768));
    assert!(has_colour(&img, |p| p[1] > 160 && p[0] < 80 && p[2] < 80));
    assert!(has_colour(&img, |p| p[0] > 160 && p[1] > 120 && p[2] < 80));
    assert!(!has_colour(&img, |p| p[0] > 160 && p[1] < 80 && p[2] < 80));
    assert!(!has_colour(&img, |p| p[2] > 160 && p[0] < 80 && p[1] < 80));
}

#[test]
fn a10_c1c_small_preview_never_upscaled() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("adjusted.jpg");
    std::fs::write(
        &source,
        psm_testkit::fixtures::synthetic_jpeg::SyntheticJpeg::rgb(800, 600, [20, 30, 40]),
    )
    .unwrap();
    for edge in [1024, 2048] {
        let dest = dir.path().join(format!("research-{edge}.jpg"));
        let info = ImageOpsImpl
            .render_research(&source, &original(), edge, &dest)
            .unwrap();
        assert_eq!((info.width, info.height), (800, 600));
        assert_eq!(image::open(&dest).unwrap().width(), 800);
        assert_eq!(image::open(&dest).unwrap().height(), 600);
    }
}

#[test]
fn a10_c1d_large_preview_resized_to_2048() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("adjusted.jpg");
    let dest = dir.path().join("research.jpg");
    std::fs::write(
        &source,
        psm_testkit::fixtures::synthetic_jpeg::SyntheticJpeg::rgb(6000, 4000, [20, 30, 40]),
    )
    .unwrap();
    let info = ImageOpsImpl
        .render_research(&source, &original(), 2048, &dest)
        .unwrap();
    assert_eq!((info.width, info.height), (2048, 1365));
    assert_eq!(image::open(&dest).unwrap().dimensions(), (2048, 1365));
}

fn preview_with_sensitive_exif(jpeg: &[u8]) -> Vec<u8> {
    use exif::{Field, In, Rational, Tag, Value, experimental::Writer};
    let fields = [
        Field {
            tag: Tag::Model,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![b"Camera Model".to_vec()]),
        },
        Field {
            tag: Tag::BodySerialNumber,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![b"SERIAL-123".to_vec()]),
        },
        Field {
            tag: Tag::MakerNote,
            ifd_num: In::PRIMARY,
            value: Value::Undefined(b"MAKER-PRIVATE".to_vec(), 0),
        },
        Field {
            tag: Tag::GPSLatitude,
            ifd_num: In::PRIMARY,
            value: Value::Rational(vec![
                Rational { num: 59, denom: 1 },
                Rational { num: 20, denom: 1 },
                Rational { num: 0, denom: 1 },
            ]),
        },
    ];
    let mut writer = Writer::new();
    for field in &fields {
        writer.push_field(field);
    }
    let mut tiff = std::io::Cursor::new(Vec::new());
    writer.write(&mut tiff, false).unwrap();
    let mut payload = b"Exif\0\0".to_vec();
    payload.extend(tiff.into_inner());
    let len = u16::try_from(payload.len() + 2).unwrap();
    let mut out = jpeg[..2].to_vec();
    out.extend([0xff, 0xe1]);
    out.extend(len.to_be_bytes());
    out.extend(payload);
    out.extend_from_slice(&jpeg[2..]);
    out
}

#[test]
fn a10_c2a_exif_and_gps_removed() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("adjusted.jpg");
    let dest = dir.path().join("research.jpg");
    std::fs::write(
        &source,
        preview_with_sensitive_exif(&marked_preview(800, 600)),
    )
    .unwrap();
    let file = std::fs::File::open(&source).unwrap();
    let original_exif = exif::Reader::new()
        .read_from_container(&mut std::io::BufReader::new(file))
        .unwrap();
    for tag in [
        exif::Tag::Model,
        exif::Tag::BodySerialNumber,
        exif::Tag::MakerNote,
        exif::Tag::GPSLatitude,
    ] {
        assert!(
            original_exif.get_field(tag, exif::In::PRIMARY).is_some(),
            "source {tag:?}"
        );
    }
    ImageOpsImpl
        .render_research(&source, &original(), 1024, &dest)
        .unwrap();
    let file = std::fs::File::open(&dest).unwrap();
    assert!(matches!(
        exif::Reader::new().read_from_container(&mut std::io::BufReader::new(file)),
        Err(exif::Error::NotFound(_))
    ));
    let bytes = std::fs::read(&dest).unwrap();
    for secret in [
        b"SERIAL-123".as_slice(),
        b"MAKER-PRIVATE".as_slice(),
        b"Camera Model".as_slice(),
    ] {
        assert!(!bytes.windows(secret.len()).any(|window| window == secret));
    }
}
