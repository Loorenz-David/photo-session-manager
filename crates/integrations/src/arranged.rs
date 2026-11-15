//! The app seams for the photographer's arranged mark and publish preview.

use psm_core::{
    Error, Result, domain::publish::PublishPreview, services::sessions::SessionsService,
};

use crate::AppHandles;

pub struct ArrangedNext {
    pub next_session_id: Option<i64>,
    pub research_queued: bool,
}

pub struct PublishResearchPreview {
    pub preview: PublishPreview,
    pub researching: Vec<bool>,
}

fn sessions(app: &AppHandles) -> SessionsService<'_> {
    SessionsService::new(app.store.as_ref(), app.clock.as_ref()).with_events(app.events.as_ref())
}

fn mark(app: &AppHandles, session_id: i64) -> Result<bool> {
    sessions(app).arrange(session_id)?;
    match &app.research {
        Some(research) => match research.arranged(session_id) {
            Ok(queued) => Ok(queued),
            Err(error) => {
                tracing::warn!(session_id, %error, "research could not be queued after arranging");
                Ok(false)
            }
        },
        None => Ok(false),
    }
}

pub fn arrange_and_next(app: &AppHandles, session_id: i64) -> Result<ArrangedNext> {
    let research_queued = mark(app, session_id)?;
    Ok(ArrangedNext {
        next_session_id: sessions(app).next_unarranged(session_id)?,
        research_queued,
    })
}

pub fn preview_for_publish(
    app: &AppHandles,
    session_ids: &[i64],
) -> Result<PublishResearchPreview> {
    for &session_id in session_ids {
        if sessions(app).get(session_id)?.session.arranged_at.is_none() {
            mark(app, session_id)?;
        }
    }
    let preview = app
        .publish
        .as_ref()
        .ok_or(Error::NoIntegration)?
        .preview(session_ids)?;
    let researching = preview
        .items
        .iter()
        .map(|item| {
            app.research
                .as_ref()
                .and_then(|research| research.status(item.session_id).ok())
                .and_then(|status| status.run)
                .is_some_and(|run| run.state == "queued" || run.state == "running")
        })
        .collect();
    Ok(PublishResearchPreview {
        preview,
        researching,
    })
}
