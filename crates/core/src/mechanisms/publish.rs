use crate::domain::publish::PublishPreview;

pub fn publish_summary(preview: &PublishPreview) -> String {
    let images: usize = preview
        .items
        .iter()
        .map(|item| item.main.len() + item.condition.len())
        .sum();
    let unreviewed: usize = preview.items.iter().map(|item| item.unreviewed).sum();
    let removals: usize = preview
        .items
        .iter()
        .map(|item| item.announced_removals.len())
        .sum();
    let creates = preview.items.iter().filter(|item| item.create_item).count();
    let mut text = format!("Publish {images} images");
    if unreviewed > 0 {
        text.push_str(&format!(
            " · {unreviewed} unreviewed photos will not be published"
        ));
    }
    if removals > 0 {
        text.push_str(&format!(" · Remove {removals} shop gallery photos"));
    }
    if creates > 0 {
        text.push_str(&format!(" · Create {creates} shop items"));
    }
    text
}
