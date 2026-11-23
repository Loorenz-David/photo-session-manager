//! "Delete session" end to end (owner correction A17): Capture One first leaves the session when
//! photos are filed into it (the same ordered switch as going back to the list, A10/A16), then the
//! storage service sends its files to the Trash and removes its rows. Capture One not running, or
//! not answering, never stops the delete: it is logged (`capture_one_result=`). Shopify is never
//! asked anything (the owner's scope: Session Manager only). The item's picture caches
//! (`previews/remote/<item>/`, `previews/archived/<item>/`) go too (H2).
//!
//! Capture One's own collection for the folder is not removed through the dictionary: its only
//! way is the generic `delete` on a session-folder collection, which the dictionary does not
//! document for folders on disk, so the app does not risk it. The folder itself goes to the Trash,
//! and Capture One no longer finds it there.

use crate::AppHandles;
use psm_core::{
    Result,
    ports::{clock::SystemClock, store::Store},
    services::{
        sessions::{CAPTURE_TARGET_ORDER, SessionsService},
        storage::{DeletePreview, SessionDeleted},
    },
};

/// What the confirm dialog shows.
pub fn preview(app: &AppHandles, session_id: i64) -> Result<DeletePreview> {
    app.storage.delete_preview(session_id)
}

pub fn delete(app: &AppHandles, session_id: i64) -> Result<SessionDeleted> {
    let preview = app.storage.delete_preview(session_id)?;
    let item_id = app.store.session(session_id)?.session.item_id;
    let capture_one = if preview.active {
        let service = || {
            SessionsService::new(app.store.as_ref(), &SystemClock)
                .with_editing(app.editing.as_ref(), app.health.as_ref(), &app.provision)
                .with_camera(app.camera.as_ref())
                .with_follow(app.follow.as_ref())
        };
        let ticket = CAPTURE_TARGET_ORDER.ticket();
        let claimed = CAPTURE_TARGET_ORDER.claim(ticket, || service().claim_target(None))?;
        match CAPTURE_TARGET_ORDER.run(ticket, "leave deleted session", || {
            service().with_target_claimed(claimed).deactivate()
        }) {
            Ok(()) => "left_session".to_owned(),
            Err(error) => {
                tracing::warn!(%error, "session.delete Capture One did not leave the session");
                format!("not_reached ({error})")
            }
        }
    } else {
        "not_needed".to_owned()
    };
    let deleted = app.storage.delete_session(session_id, &capture_one)?;
    // H2 (M-H5, 10H §9): the item's picture caches on this Mac go with its row; no shop call.
    if let Some(item_id) = item_id
        && app.store.item(item_id).is_err()
        && let Err(error) = app.remote.forget_item(item_id)
    {
        tracing::warn!(%error, item = item_id, "session.delete picture caches kept");
    }
    Ok(deleted)
}
