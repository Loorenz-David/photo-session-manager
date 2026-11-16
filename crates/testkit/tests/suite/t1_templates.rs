use psm_core::{
    Error,
    domain::{
        composition::Shape,
        template::{Presets, Template},
    },
    services::templates::TemplatesService,
};
use psm_infra::imaging::ImageOpsImpl;
use psm_testkit::TestApp;
use std::{
    fs,
    path::{Path, PathBuf},
};

fn service(a: &TestApp) -> TemplatesService<'_> {
    TemplatesService::new(
        a.store(),
        a.clock.as_ref(),
        &ImageOpsImpl,
        a.app.paths.root.join("templates"),
    )
}
fn image(a: &TestApp, name: &str, width: u32, height: u32) -> PathBuf {
    let path = a.app.paths.root.join(name);
    image::RgbImage::from_pixel(width, height, image::Rgb([120, 90, 50]))
        .save(&path)
        .unwrap();
    path
}
fn three(a: &TestApp) -> Template {
    let s = service(a);
    let t = s.create("Dining chair").unwrap();
    let t = s
        .container_add(t.id, Some(&image(a, "first.jpg", 3000, 2000)))
        .unwrap();
    let t = s
        .container_add(t.id, Some(&image(a, "second.png", 100, 80)))
        .unwrap();
    s.container_add(t.id, None).unwrap()
}
fn ids(t: &Template) -> Vec<i64> {
    t.containers.iter().map(|c| c.id).collect()
}
fn code(error: Error) -> &'static str {
    match error {
        Error::TemplateExists => "template_exists",
        Error::TemplateImageUnreadable => "template_image_unreadable",
        Error::InvalidValue(_) => "invalid_value",
        _ => panic!("wrong error: {error}"),
    }
}
fn exists(path: &Option<String>) -> bool {
    path.as_ref().is_some_and(|p| Path::new(p).exists())
}

#[test]
fn t1_c1a_create_and_lookup() {
    let a = TestApp::new().unwrap();
    let s = service(&a);
    let t = s.create("Dining chair").unwrap();
    assert!(t.containers.is_empty());
    assert_eq!(s.get("dining chair ").unwrap().unwrap().id, t.id);
    let list = s.list().unwrap();
    assert_eq!(
        (
            list.len(),
            list[0].item_type.as_str(),
            list[0].container_count
        ),
        (1, "Dining chair", 0)
    );
}
#[test]
fn t1_c1b_duplicate_type() {
    let a = TestApp::new().unwrap();
    let s = service(&a);
    s.create("Dining chair").unwrap();
    assert_eq!(
        code(s.create("DINING CHAIR").unwrap_err()),
        "template_exists"
    );
    assert_eq!(s.list().unwrap().len(), 1);
}
#[test]
fn t1_c1c_delete_template_and_examples() {
    let a = TestApp::new().unwrap();
    let initial = three(&a);
    let s = service(&a);
    let t = s.container_remove(initial.containers[2].id).unwrap();
    assert!(exists(&t.containers[0].example_path));
    s.delete(t.id).unwrap();
    assert!(s.get("Dining chair").unwrap().is_none());
    assert!(s.list().unwrap().is_empty());
    assert!(
        !a.app
            .paths
            .root
            .join("templates")
            .join(t.id.to_string())
            .exists()
    );
}
#[test]
fn t1_c2a_add_examples_and_empty_container() {
    let a = TestApp::new().unwrap();
    let jpeg = image(&a, "first.jpg", 3000, 2000);
    let png = image(&a, "second.png", 100, 80);
    let jpeg_before = fs::read(&jpeg).unwrap();
    let png_before = fs::read(&png).unwrap();
    let s = service(&a);
    let t = s.create("Dining chair").unwrap();
    let t = s.container_add(t.id, Some(&jpeg)).unwrap();
    let t = s.container_add(t.id, Some(&png)).unwrap();
    let t = s.container_add(t.id, None).unwrap();
    assert_eq!(
        t.containers.iter().map(|c| c.final_no).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(
        t.containers.iter().map(|c| c.shoot_no).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    for c in &t.containers[..2] {
        let path = c.example_path.as_ref().unwrap();
        assert!(
            path.starts_with(
                a.app
                    .paths
                    .root
                    .join("templates")
                    .join(t.id.to_string())
                    .to_str()
                    .unwrap()
            )
        );
        let reader = image::ImageReader::open(path)
            .unwrap()
            .with_guessed_format()
            .unwrap();
        assert_eq!(reader.format(), Some(image::ImageFormat::Jpeg));
        assert!(reader.decode().is_ok());
        let (w, h) = image::image_dimensions(path).unwrap();
        assert!(w.max(h) <= 1600);
    }
    assert!(t.containers[2].example_file.is_none());
    assert_eq!(fs::read(jpeg).unwrap(), jpeg_before);
    assert_eq!(fs::read(png).unwrap(), png_before);
}
#[test]
fn t1_c2b_unreadable_does_not_add() {
    let a = TestApp::new().unwrap();
    let t = three(&a);
    let source = a.app.paths.root.join("x.jpg");
    fs::write(&source, "not an image").unwrap();
    let s = service(&a);
    assert_eq!(
        code(s.container_add(t.id, Some(&source)).unwrap_err()),
        "template_image_unreadable"
    );
    assert_eq!(s.get("Dining chair").unwrap().unwrap().containers.len(), 3);
    assert_eq!(
        fs::read_dir(a.app.paths.root.join("templates").join(t.id.to_string()))
            .unwrap()
            .count(),
        2
    );
}
#[test]
fn t1_c2c_final_order_does_not_change_shoot_order() {
    let a = TestApp::new().unwrap();
    let t = three(&a);
    let [c1, c2, c3] = ids(&t)[..] else {
        unreachable!()
    };
    let after = service(&a).containers_reorder(t.id, &[c3, c1, c2]).unwrap();
    assert_eq!(ids(&after), vec![c3, c1, c2]);
    assert_eq!(
        after
            .containers
            .iter()
            .map(|c| c.shoot_no)
            .collect::<Vec<_>>(),
        vec![3, 1, 2]
    );
}
#[test]
fn t1_c2d_rejects_bad_permutations() {
    let a = TestApp::new().unwrap();
    let t = three(&a);
    let id = ids(&t);
    let s = service(&a);
    assert_eq!(
        code(s.containers_reorder(t.id, &id[..2]).unwrap_err()),
        "invalid_value"
    );
    assert_eq!(
        code(
            s.shooting_order_set(t.id, &[id[0], id[0], id[1]])
                .unwrap_err()
        ),
        "invalid_value"
    );
    assert_eq!(s.get("Dining chair").unwrap().unwrap(), t);
}
#[test]
fn t1_c2e_shooting_order_does_not_change_final_order() {
    let a = TestApp::new().unwrap();
    let t = three(&a);
    let [c1, c2, c3] = ids(&t)[..] else {
        unreachable!()
    };
    let s = service(&a);
    s.containers_reorder(t.id, &[c3, c1, c2]).unwrap();
    let after = s.shooting_order_set(t.id, &[c2, c3, c1]).unwrap();
    assert_eq!(ids(&after), vec![c3, c1, c2]);
    assert_eq!(
        after
            .containers
            .iter()
            .map(|c| c.shoot_no)
            .collect::<Vec<_>>(),
        vec![2, 3, 1]
    );
}
#[test]
fn t1_c2f_remove_renumbers_both_orders() {
    let a = TestApp::new().unwrap();
    let t = three(&a);
    let [c1, c2, c3] = ids(&t)[..] else {
        unreachable!()
    };
    let s = service(&a);
    let old_path = t.containers[0].example_path.clone().unwrap();
    s.containers_reorder(t.id, &[c3, c1, c2]).unwrap();
    s.shooting_order_set(t.id, &[c2, c3, c1]).unwrap();
    let after = s.container_remove(c1).unwrap();
    assert_eq!(ids(&after), vec![c3, c2]);
    assert_eq!(
        after
            .containers
            .iter()
            .map(|c| c.final_no)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(
        after
            .containers
            .iter()
            .map(|c| c.shoot_no)
            .collect::<Vec<_>>(),
        vec![2, 1]
    );
    assert!(!Path::new(&old_path).exists());
}
#[test]
fn t1_c2g_presets_and_invalid_values() {
    let a = TestApp::new().unwrap();
    let t = three(&a);
    let c3 = t.containers[2].id;
    let s = service(&a);
    let wanted = Presets {
        split: true,
        shape: Some(Shape::FourFive),
        zoom: 20,
    };
    let after = s.container_presets_set(c3, wanted.clone()).unwrap();
    assert_eq!(after.containers[2].presets, wanted);
    assert_eq!(after.containers[1].presets, Presets::default());
    assert_eq!(
        code(
            s.container_presets_set(
                c3,
                Presets {
                    zoom: 81,
                    ..wanted.clone()
                }
            )
            .unwrap_err()
        ),
        "invalid_value"
    );
    assert_eq!(
        code(
            s.container_presets_set(
                c3,
                Presets {
                    shape: Some(Shape::Free),
                    ..wanted.clone()
                }
            )
            .unwrap_err()
        ),
        "invalid_value"
    );
    assert_eq!(
        s.get("Dining chair").unwrap().unwrap().containers[2].presets,
        wanted
    );
}
#[test]
fn t1_c2h_replace_example() {
    let a = TestApp::new().unwrap();
    let t = three(&a);
    let old = t.containers[0].example_path.clone().unwrap();
    let replacement = image(&a, "replacement.jpg", 50, 50);
    let after = service(&a)
        .container_example_set(t.containers[0].id, &replacement)
        .unwrap();
    assert!(exists(&after.containers[0].example_path));
    assert_ne!(
        after.containers[0].example_path.as_deref(),
        Some(old.as_str())
    );
    assert!(!Path::new(&old).exists());
}
#[test]
fn t1_c3a_restart_keeps_template() {
    let mut a = TestApp::new().unwrap();
    let t = three(&a);
    let ids = ids(&t);
    let s = service(&a);
    s.containers_reorder(t.id, &[ids[2], ids[0], ids[1]])
        .unwrap();
    s.shooting_order_set(t.id, &[ids[1], ids[2], ids[0]])
        .unwrap();
    let saved = s
        .container_presets_set(
            ids[2],
            Presets {
                split: true,
                shape: Some(Shape::FourFive),
                zoom: 20,
            },
        )
        .unwrap();
    a.kill_and_restart().unwrap();
    let loaded = service(&a).get("dining chair").unwrap().unwrap();
    assert_eq!(loaded, saved);
    assert!(
        loaded
            .containers
            .iter()
            .filter(|c| c.example_file.is_some())
            .all(|c| exists(&c.example_path))
    );
}
