use crate::domain::session::{PublishStep, RowLabel, SessionStatus};

#[derive(Debug, Clone, Copy)]
pub struct StatusInput {
    pub latest_step: Option<PublishStep>,
    pub failed_step: Option<PublishStep>,
    pub published_at: Option<i64>,
    pub dirty: bool,
    pub main_count: i64,
    pub unreviewed_count: i64,
    pub non_deleted_missing_preview_count: i64,
    pub uploaded: i64,
    pub total: i64,
}

pub fn derive_status(input: StatusInput) -> SessionStatus {
    if matches!(
        input.latest_step,
        Some(
            PublishStep::Prepare
                | PublishStep::Upload
                | PublishStep::Update
                | PublishStep::Describe
        )
    ) {
        SessionStatus::Publishing
    } else if input.latest_step == Some(PublishStep::Failed) && input.published_at.is_none() {
        SessionStatus::Failed
    } else if input.published_at.is_some() && !input.dirty {
        SessionStatus::Published
    } else if input.main_count > 0
        && input.unreviewed_count == 0
        && input.non_deleted_missing_preview_count == 0
    {
        SessionStatus::Ready
    } else {
        SessionStatus::InProgress
    }
}

pub fn progress_percent(step: PublishStep, uploaded: i64, total: i64) -> u8 {
    match step {
        PublishStep::Prepare => 0,
        PublishStep::Upload => {
            let fraction = if total > 0 {
                65 * uploaded.clamp(0, total) / total
            } else {
                0
            };
            (10 + fraction) as u8
        }
        PublishStep::Update => 75,
        PublishStep::Describe => 85,
        PublishStep::Done => 100,
        PublishStep::Failed => 0,
    }
}

pub fn row_label(input: StatusInput) -> RowLabel {
    let status = derive_status(input);
    if status == SessionStatus::Publishing {
        RowLabel::Publishing(progress_percent(
            input.latest_step.unwrap(),
            input.uploaded,
            input.total,
        ))
    } else if input.latest_step == Some(PublishStep::Failed) {
        if input.failed_step == Some(PublishStep::Describe) {
            RowLabel::DescriptionFailed
        } else {
            RowLabel::PublishFailed
        }
    } else {
        match status {
            SessionStatus::Published => RowLabel::Published,
            SessionStatus::Ready => RowLabel::Ready,
            _ => RowLabel::InProgress(input.unreviewed_count),
        }
    }
}
