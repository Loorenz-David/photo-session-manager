use crate::domain::{
    HealthItem,
    camera::{BusyReason, CameraName, CameraStatus, Link, NeedsActionReason},
};

/// One reason that needs the photographer: `(reason, holder, message)`; `message` is the MC-34
/// sentence for `HeldByOtherProgram` (P48 R-I30), `None` for every other reason.
pub type NeedsActionEntry = (NeedsActionReason, Option<String>, Option<String>);

pub struct StatusInputs {
    pub needs_action: Vec<NeedsActionEntry>,
    pub busy: Option<BusyReason>,
    pub session: Option<(CameraName, Link)>,
    pub connecting: Option<Link>,
    pub helper_running: bool,
}

pub fn derive_status(input: &StatusInputs) -> CameraStatus {
    if let Some((reason, holder, message)) =
        input.needs_action.iter().min_by_key(|(r, _, _)| r.rank())
    {
        return CameraStatus::NeedsAction {
            reason: reason.clone(),
            holder: holder.clone(),
            message: message.clone(),
        };
    }
    if let Some(reason) = &input.busy {
        return CameraStatus::Busy {
            reason: reason.clone(),
        };
    }
    if let Some((name, link)) = &input.session {
        return CameraStatus::Connected {
            name: name.0.clone(),
            link: link.clone(),
        };
    }
    if input.helper_running
        && let Some(link) = &input.connecting
    {
        return CameraStatus::Connecting { link: link.clone() };
    }
    CameraStatus::Disconnected
}

pub fn status_message(reason: &NeedsActionReason, holder: Option<&str>) -> String {
    match reason {
        NeedsActionReason::AuthenticationRefused => "The camera refused the password".into(),
        NeedsActionReason::FingerprintMismatch => "The camera's fingerprint changed: confirm it in Settings → Camera before shooting over Wi-Fi".into(),
        NeedsActionReason::FingerprintUnconfirmed => "Confirm the camera's fingerprint".into(),
        NeedsActionReason::PhoneAppConnected => "A phone app is connected to the camera: disconnect it".into(),
        NeedsActionReason::HeldByOtherProgram => holder.map(|h| format!("{h} is holding the camera")).unwrap_or_else(|| "Another program is holding the camera".into()),
        NeedsActionReason::StoreDestinationUnverified => "The camera did not confirm where photos are saved".into(),
        NeedsActionReason::FilesNotReceived { count } => format!("{count} photos were taken but not received. The camera may have stopped sending: on the camera, switch FTP Function off and on."),
        NeedsActionReason::SupportNotInstalled => "Add camera support in Settings → Camera".into(),
    }
}

pub fn health_items(input: &StatusInputs) -> Vec<HealthItem> {
    let mut reasons = input.needs_action.clone();
    reasons.sort_by_key(|(r, _, _)| r.rank());
    let mut items: Vec<_> = reasons
        .iter()
        .map(|(r, holder, message)| HealthItem {
            code: r.code().into(),
            ok: false,
            detail: message
                .clone()
                .unwrap_or_else(|| status_message(r, holder.as_deref())),
        })
        .collect();
    let (code, ok, detail) = match derive_status(input) {
        CameraStatus::Connected { name, link } => {
            ("camera", true, format!("{name} · {}", link.label()))
        }
        CameraStatus::Connecting { link } => (
            "camera_connecting",
            false,
            match link {
                Link::Cable => "Connecting over the cable…",
                Link::Wireless => "Connecting over Wi-Fi…",
            }
            .into(),
        ),
        CameraStatus::Busy { reason } => (
            "camera_busy",
            false,
            match reason {
                BusyReason::Exposure => "Taking a photo",
                BusyReason::Transfer => "Receiving a photo",
                BusyReason::Switching => "Switching connection",
            }
            .into(),
        ),
        CameraStatus::NeedsAction { .. } if input.session.is_some() => {
            let (name, link) = input.session.as_ref().expect("checked");
            ("camera", true, format!("{} · {}", name.0, link.label()))
        }
        _ => ("camera_disconnected", false, "No camera connected".into()),
    };
    items.push(HealthItem {
        code: code.into(),
        ok,
        detail,
    });
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    fn empty() -> StatusInputs {
        StatusInputs {
            needs_action: vec![],
            busy: None,
            session: None,
            connecting: None,
            helper_running: false,
        }
    }
    macro_rules! rank_case {
        ($name:ident, $lower:expr, $higher:expr) => {
            #[test]
            fn $name() {
                let mut input = empty();
                let higher = $higher;
                input.needs_action = vec![($lower, None, None), (higher.clone(), None, None)];
                assert_eq!(
                    derive_status(&input),
                    CameraStatus::NeedsAction {
                        reason: higher,
                        holder: None,
                        message: None
                    }
                );
            }
        };
    }
    use NeedsActionReason::*;
    rank_case!(
        mc31_c5a_authentication_refused,
        FingerprintMismatch,
        AuthenticationRefused
    );
    rank_case!(
        mc31_c5b_fingerprint_mismatch,
        FingerprintUnconfirmed,
        FingerprintMismatch
    );
    rank_case!(
        mc31_c5c_fingerprint_unconfirmed,
        PhoneAppConnected,
        FingerprintUnconfirmed
    );
    rank_case!(
        mc31_c5d_phone_app_connected,
        HeldByOtherProgram,
        PhoneAppConnected
    );
    rank_case!(
        mc31_c5e_held_by_other_program,
        StoreDestinationUnverified,
        HeldByOtherProgram
    );
    rank_case!(
        mc31_c5f_store_destination,
        FilesNotReceived { count: 2 },
        StoreDestinationUnverified
    );
    rank_case!(
        mc31_c5g_files_not_received,
        SupportNotInstalled,
        FilesNotReceived { count: 2 }
    );
    #[test]
    fn mc31_c5h_needs_action_over_busy() {
        let mut input = empty();
        input.needs_action.push((
            NeedsActionReason::HeldByOtherProgram,
            Some("ptpcamerad".into()),
            None,
        ));
        input.busy = Some(BusyReason::Exposure);
        input.session = Some((CameraName("X".into()), Link::Wireless));
        assert_eq!(
            derive_status(&input),
            CameraStatus::NeedsAction {
                reason: NeedsActionReason::HeldByOtherProgram,
                holder: Some("ptpcamerad".into()),
                message: None
            }
        );
    }
    #[test]
    fn mc31_c5i_busy_over_connected() {
        let mut input = empty();
        input.busy = Some(BusyReason::Transfer);
        input.session = Some((CameraName("X".into()), Link::Cable));
        assert_eq!(
            derive_status(&input),
            CameraStatus::Busy {
                reason: BusyReason::Transfer
            }
        );
    }
    #[test]
    fn mc31_c5j_connected_over_connecting() {
        let mut input = empty();
        input.session = Some((CameraName("X".into()), Link::Cable));
        input.connecting = Some(Link::Wireless);
        assert_eq!(
            derive_status(&input),
            CameraStatus::Connected {
                name: "X".into(),
                link: Link::Cable
            }
        );
    }
    #[test]
    fn mc31_c5k_connecting_requires_report() {
        let mut input = empty();
        input.helper_running = true;
        input.connecting = Some(Link::Wireless);
        assert_eq!(
            derive_status(&input),
            CameraStatus::Connecting {
                link: Link::Wireless
            }
        );
        input.connecting = None;
        assert_eq!(derive_status(&input), CameraStatus::Disconnected);
        input.helper_running = false;
        input.connecting = Some(Link::Wireless);
        assert_eq!(derive_status(&input), CameraStatus::Disconnected);
    }
    #[test]
    fn mc31_health_list() {
        let mut input = empty();
        input.needs_action = vec![
            (NeedsActionReason::FilesNotReceived { count: 3 }, None, None),
            (NeedsActionReason::FingerprintUnconfirmed, None, None),
        ];
        input.session = Some((CameraName("X".into()), Link::Wireless));
        assert_eq!(
            derive_status(&input),
            CameraStatus::NeedsAction {
                reason: NeedsActionReason::FingerprintUnconfirmed,
                holder: None,
                message: None
            }
        );
        let items = health_items(&input);
        assert_eq!(
            items
                .iter()
                .map(|i| (i.code.as_str(), i.ok, i.detail.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (
                    "camera_fingerprint_unconfirmed",
                    false,
                    "Confirm the camera's fingerprint"
                ),
                (
                    "camera_files_not_received",
                    false,
                    "3 photos were taken but not received. The camera may have stopped sending: on the camera, switch FTP Function off and on."
                ),
                ("camera", true, "X · wireless")
            ]
        );
    }
}
