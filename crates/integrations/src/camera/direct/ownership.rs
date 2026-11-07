//! Naming who holds the camera (10E MC-34 §Detect, §Guide; R-I30). macOS names only the process
//! that owns the USB interface; when that is the system camera service, the programs that could
//! have asked it are named from a fixed list, Capture One first (V6: it is the trigger).
use serde::Serialize;

/// The programs that use macOS camera services (R-I30), in naming order.
pub const KNOWN_CAMERA_PROGRAMS: [&str; 6] = [
    "Capture One",
    "Imaging Edge Desktop",
    "Image Capture",
    "Photos",
    "Adobe Lightroom Classic",
    "Adobe Lightroom",
];

/// The system camera service's process name.
pub const SYSTEM_CAMERA_SERVICE: &str = "ptpcamerad";

/// Who holds the camera and what the photographer can do about it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HolderReport {
    /// The first named program, else the owner process (`NeedsAction.holder`).
    pub holder: Option<String>,
    /// The known programs named behind the system camera service, in list order.
    pub programs: Vec<String>,
    /// The MC-34 sentence (`camera_held` detail, `NeedsAction.message`).
    pub sentence: Option<String>,
}

/// The known camera programs among `running`, in `KNOWN_CAMERA_PROGRAMS` order.
pub fn known_running(running: &[String]) -> Vec<String> {
    KNOWN_CAMERA_PROGRAMS
        .iter()
        .filter(|known| running.iter().any(|r| r == *known))
        .map(|known| (*known).to_owned())
        .collect()
}

fn guide(program: &str) -> &'static str {
    match program {
        "Imaging Edge Desktop" => "quit it and stop it opening at login",
        "Image Capture" | "Photos" => "set \"Connecting this camera opens\" to \"No application\"",
        _ => "quit it",
    }
}

/// R-I30: the report for the interface owner `owner_process` (`holder_from_owners(owners)`) with
/// the process names `running`.
pub fn name_programs(owner_process: &str, running: &[String]) -> HolderReport {
    if owner_process != SYSTEM_CAMERA_SERVICE {
        return HolderReport {
            holder: Some(owner_process.to_owned()),
            programs: Vec::new(),
            sentence: Some(format!(
                "{owner_process} is using the camera: close it or stop it from using cameras"
            )),
        };
    }
    let programs = known_running(running);
    let sentence = match programs.first() {
        Some(first) if first == "Capture One" => {
            "Capture One is holding the camera's USB connection — Take the camera back".to_owned()
        }
        Some(first) => format!(
            "programs that can take the camera: {} — {}",
            programs.join(", "),
            guide(first)
        ),
        None => "The system camera service is holding the camera: close apps that use cameras"
            .to_owned(),
    };
    HolderReport {
        holder: Some(
            programs
                .first()
                .cloned()
                .unwrap_or_else(|| owner_process.to_owned()),
        ),
        programs,
        sentence: Some(sentence),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn c1a_capture_one_first() {
        let r = name_programs(
            "ptpcamerad",
            &names(&["Finder", "Imaging Edge Desktop", "Capture One"]),
        );
        assert_eq!(r.programs, names(&["Capture One", "Imaging Edge Desktop"]));
        assert_eq!(r.holder.as_deref(), Some("Capture One"));
        assert_eq!(
            r.sentence.as_deref(),
            Some("Capture One is holding the camera's USB connection — Take the camera back")
        );
    }

    #[test]
    fn c1b_guides_for_other_programs() {
        assert_eq!(
            name_programs("ptpcamerad", &names(&["Photos"]))
                .sentence
                .as_deref(),
            Some(
                "programs that can take the camera: Photos — set \"Connecting this camera opens\" to \"No application\""
            )
        );
        assert_eq!(
            name_programs(
                "ptpcamerad",
                &names(&["Imaging Edge Desktop", "Adobe Lightroom"])
            )
            .sentence
            .as_deref(),
            Some(
                "programs that can take the camera: Imaging Edge Desktop, Adobe Lightroom — quit it and stop it opening at login"
            )
        );
    }

    #[test]
    fn c1c_system_service_alone() {
        let r = name_programs("ptpcamerad", &[]);
        assert_eq!(r.programs, Vec::<String>::new());
        assert_eq!(r.holder.as_deref(), Some("ptpcamerad"));
        assert_eq!(
            r.sentence.as_deref(),
            Some("The system camera service is holding the camera: close apps that use cameras")
        );
    }

    #[test]
    fn c1d_unknown_owner() {
        let r = name_programs("SomeApp", &names(&["Capture One"]));
        assert_eq!(r.programs, Vec::<String>::new());
        assert_eq!(r.holder.as_deref(), Some("SomeApp"));
        assert_eq!(
            r.sentence.as_deref(),
            Some("SomeApp is using the camera: close it or stop it from using cameras")
        );
    }
}
