//! Camera re-send suffixes (10E MC-33 R-I26): camera-specific, so the adapter strips them before
//! the ledger keys a file. Core strips only its own `__dupN`.

/// Sony: a `_<1–2 digits>` suffix after the stem's trailing digits is a re-send counter and is
/// removed (`DSC01234_1.ARW` → `DSC01234.ARW`). Anything else is returned unchanged
/// (`IMG_1234.CR3`, `DSC01234_123.ARW`, `DSC01234__dup1.ARW`).
pub fn normalize_camera_filename(name: &str) -> String {
    let (stem, extension) = match name.rfind('.') {
        Some(at) if at > 0 => (&name[..at], &name[at..]),
        _ => (name, ""),
    };
    if let Some(at) = stem.rfind('_') {
        let (head, suffix) = (&stem[..at], &stem[at + 1..]);
        if (1..=2).contains(&suffix.len())
            && suffix.bytes().all(|b| b.is_ascii_digit())
            && head.bytes().last().is_some_and(|b| b.is_ascii_digit())
        {
            return format!("{head}{extension}");
        }
    }
    name.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c2d_sony_resend_suffix() {
        for (input, expected) in [
            ("DSC01234_1.ARW", "DSC01234.ARW"),
            ("DSC01234_12.ARW", "DSC01234.ARW"),
            ("DSC01234_123.ARW", "DSC01234_123.ARW"),
            ("DSC01234.ARW", "DSC01234.ARW"),
            ("IMG_1234.CR3", "IMG_1234.CR3"),
            ("DSC01234__dup1.ARW", "DSC01234__dup1.ARW"),
        ] {
            assert_eq!(normalize_camera_filename(input), expected, "{input}");
        }
    }
}
