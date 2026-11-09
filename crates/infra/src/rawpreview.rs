use psm_core::{
    Result,
    domain::composition::Frame,
    ports::raw_previews::{Extracted, RawPreviews},
};
use std::{collections::HashSet, fs, path::Path};

pub struct EmbeddedJpeg;
impl RawPreviews for EmbeddedJpeg {
    fn extract(&self, raw: &Path, dest: &Path) -> Result<Option<Extracted>> {
        let bytes = fs::read(raw)?;
        let Some((offset, len, width, height)) = largest_embedded_jpeg(&bytes) else {
            return Ok(None);
        };
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp = dest.with_extension("jpg.tmp");
        fs::write(&temp, &bytes[offset..offset + len])?;
        fs::rename(temp, dest)?;
        let orientation = orientation(&bytes);
        Ok(Some(Extracted {
            orientation,
            compose_frame: compose_frame((width, height), raw_frame(&bytes), orientation),
        }))
    }
}

/// The RAW's sensor frame (10A MC-20, P58): the DNG `DefaultCropSize` (0xC620) that Sony writes
/// in the raw SubIFD, else the largest ImageWidth × ImageLength of an IFD that carries no embedded
/// JPEG. Sony's tiled raw data pads ImageWidth/ImageLength to whole tiles (A7R V: 6656 × 4608 for
/// a 6240 × 4160 frame), so the crop size wins whenever it exists. `None` fails closed.
pub fn raw_frame(bytes: &[u8]) -> Option<(u32, u32)> {
    let mut crop: Option<(u32, u32)> = None;
    let mut image: Option<(u32, u32)> = None;
    let larger = |best: &mut Option<(u32, u32)>, size: (u32, u32)| {
        if size.0 > 0
            && size.1 > 0
            && best.is_none_or(|(w, h)| size.0 as u64 * size.1 as u64 > w as u64 * h as u64)
        {
            *best = Some(size);
        }
    };
    walk_ifds(bytes, |tags, little| {
        let find = |tag: u16| tags.iter().find(|t| t.0 == tag).copied();
        let single = |tag: u16| {
            find(tag).and_then(|(_, kind, count, value)| match (kind, count) {
                (3, 1) => Some(if little { value & 0xffff } else { value >> 16 }),
                (4, 1) => Some(value),
                _ => None,
            })
        };
        if let Some(size) = find(0xc620).and_then(|tag| pair(bytes, tag, little)) {
            larger(&mut crop, size);
        }
        if find(0x0201).is_none()
            && let (Some(w), Some(h)) = (single(0x0100), single(0x0101))
        {
            larger(&mut image, (w, h));
        }
    });
    crop.or(image)
}

/// A two-value TIFF field (SHORT inline, LONG or RATIONAL at an offset).
fn pair(
    bytes: &[u8],
    (_, kind, count, value): (u16, u16, u32, u32),
    little: bool,
) -> Option<(u32, u32)> {
    if count != 2 {
        return None;
    }
    let at = value as usize;
    match kind {
        3 if little => Some((value & 0xffff, value >> 16)),
        3 => Some((value >> 16, value & 0xffff)),
        4 => Some((u32_at(bytes, at, little)?, u32_at(bytes, at + 4, little)?)),
        5 => {
            let ratio = |at: usize| -> Option<u32> {
                let (n, d) = (u32_at(bytes, at, little)?, u32_at(bytes, at + 4, little)?);
                (d > 0).then(|| (n as f64 / d as f64).round() as u32)
            };
            Some((ratio(at)?, ratio(at + 8)?))
        }
        _ => None,
    }
}

/// "The original matches the camera frame" (book 10D, changelog 39): the embedded JPEG has the
/// RAW sensor frame's aspect ratio within 1 % and at least 1600 px on its long edge. The answer is
/// the JPEG's frame after the RAW's orientation, the frame a composition on it is measured in.
pub fn compose_frame(
    jpeg: (u32, u32),
    raw: Option<(u32, u32)>,
    orientation: Option<u8>,
) -> Option<Frame> {
    let (w, h) = jpeg;
    let (rw, rh) = raw?;
    if w == 0 || h == 0 || rw == 0 || rh == 0 || w.max(h) < 1600 {
        return None;
    }
    let ratio = (w as f64 / h as f64) / (rw as f64 / rh as f64);
    if (ratio - 1.0).abs() > 0.01 {
        return None;
    }
    let (w, h) = if matches!(orientation, Some(5..=8)) {
        (h, w)
    } else {
        (w, h)
    };
    Some(Frame {
        w: w as i64,
        h: h as i64,
    })
}

fn endian(bytes: &[u8]) -> Option<bool> {
    match bytes.get(0..2)? {
        b"II" => Some(true),
        b"MM" => Some(false),
        _ => None,
    }
}
fn u16_at(bytes: &[u8], at: usize, little: bool) -> Option<u16> {
    let chunk: [u8; 2] = bytes.get(at..at + 2)?.try_into().ok()?;
    Some(if little {
        u16::from_le_bytes(chunk)
    } else {
        u16::from_be_bytes(chunk)
    })
}
fn u32_at(bytes: &[u8], at: usize, little: bool) -> Option<u32> {
    let chunk: [u8; 4] = bytes.get(at..at + 4)?.try_into().ok()?;
    Some(if little {
        u32::from_le_bytes(chunk)
    } else {
        u32::from_be_bytes(chunk)
    })
}
fn walk_ifds(bytes: &[u8], mut visit: impl FnMut(&[(u16, u16, u32, u32)], bool)) {
    let Some(little) = endian(bytes) else {
        return;
    };
    if u16_at(bytes, 2, little) != Some(42) {
        return;
    }
    let mut stack = vec![u32_at(bytes, 4, little).unwrap_or(0) as usize];
    let mut seen = HashSet::new();
    while let Some(offset) = stack.pop() {
        if offset == 0 || !seen.insert(offset) || seen.len() > 128 {
            continue;
        }
        let Some(count) = u16_at(bytes, offset, little) else {
            continue;
        };
        let mut tags = Vec::new();
        for index in 0..count as usize {
            let at = offset + 2 + index * 12;
            let (Some(tag), Some(kind), Some(n), Some(value)) = (
                u16_at(bytes, at, little),
                u16_at(bytes, at + 2, little),
                u32_at(bytes, at + 4, little),
                u32_at(bytes, at + 8, little),
            ) else {
                break;
            };
            tags.push((tag, kind, n, value));
            if tag == 0x014a && kind == 4 {
                if n == 1 {
                    stack.push(value as usize);
                } else {
                    for i in 0..n.min(128) as usize {
                        if let Some(p) = u32_at(bytes, value as usize + i * 4, little) {
                            stack.push(p as usize);
                        }
                    }
                }
            }
        }
        visit(&tags, little);
        if let Some(next) = u32_at(bytes, offset + 2 + count as usize * 12, little) {
            stack.push(next as usize);
        }
    }
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if !bytes.starts_with(&[0xff, 0xd8]) {
        return None;
    }
    let mut at = 2;
    while at + 4 <= bytes.len() {
        if bytes[at] != 0xff {
            at += 1;
            continue;
        }
        let marker = bytes[at + 1];
        at += 2;
        if marker == 0xd9 || marker == 0xda {
            break;
        }
        if matches!(marker, 0xd0..=0xd7 | 0x01) {
            continue;
        }
        let size = u16::from_be_bytes(bytes.get(at..at + 2)?.try_into().ok()?) as usize;
        if size < 2 || at + size > bytes.len() {
            return None;
        }
        if matches!(marker, 0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf) && size >= 7 {
            let h = u16::from_be_bytes(bytes.get(at + 3..at + 5)?.try_into().ok()?) as u32;
            let w = u16::from_be_bytes(bytes.get(at + 5..at + 7)?.try_into().ok()?) as u32;
            return Some((w, h));
        }
        at += size;
    }
    None
}
pub fn largest_embedded_jpeg(bytes: &[u8]) -> Option<(usize, usize, u32, u32)> {
    let mut best: Option<(usize, usize, u32, u32)> = None;
    walk_ifds(bytes, |tags, _| {
        let offset = tags.iter().find(|t| t.0 == 0x0201).map(|t| t.3 as usize);
        let len = tags.iter().find(|t| t.0 == 0x0202).map(|t| t.3 as usize);
        if let (Some(offset), Some(len)) = (offset, len)
            && let Some(stream) = offset
                .checked_add(len)
                .and_then(|end| bytes.get(offset..end))
            && stream.starts_with(&[0xff, 0xd8])
            && stream
                .iter()
                .rposition(|b| *b != 0)
                .is_some_and(|end| end > 0 && stream[end - 1..=end] == [0xff, 0xd9])
            && let Some((w, h)) = jpeg_dimensions(stream)
            && best.is_none_or(|(_, _, bw, bh)| w as u64 * h as u64 > bw as u64 * bh as u64)
        {
            best = Some((offset, len, w, h));
        }
    });
    best
}
pub fn orientation(bytes: &[u8]) -> Option<u8> {
    let mut answer = None;
    let mut first = true;
    walk_ifds(bytes, |tags, little| {
        if first {
            answer = tags
                .iter()
                .find(|t| t.0 == 0x0112 && t.1 == 3 && t.2 == 1)
                .and_then(|t| {
                    let value = if little {
                        t.3 as u16
                    } else {
                        (t.3 >> 16) as u16
                    };
                    (1..=8).contains(&value).then_some(value as u8)
                });
            first = false;
        }
    });
    answer
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mc20_subifd_jpeg_and_ifd0_orientation() {
        let mut raw = vec![0u8; 100];
        raw[..8].copy_from_slice(&[b'I', b'I', 42, 0, 8, 0, 0, 0]);
        raw[8..10].copy_from_slice(&2u16.to_le_bytes());
        raw[10..22].copy_from_slice(&[0x12, 0x01, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0]);
        raw[22..34].copy_from_slice(&[0x4a, 0x01, 4, 0, 1, 0, 0, 0, 64, 0, 0, 0]);
        raw[64..66].copy_from_slice(&2u16.to_le_bytes());
        raw[66..78].copy_from_slice(&[1, 2, 4, 0, 1, 0, 0, 0, 100, 0, 0, 0]);
        let jpeg = [
            0xff, 0xd8, 0xff, 0xc0, 0, 11, 8, 0, 120, 0, 160, 1, 1, 0x11, 0, 0xff, 0xd9,
        ];
        raw[78..90].copy_from_slice(&[2, 2, 4, 0, 1, 0, 0, 0, jpeg.len() as u8, 0, 0, 0]);
        raw.extend_from_slice(&jpeg);
        assert_eq!(
            largest_embedded_jpeg(&raw),
            Some((100, jpeg.len(), 160, 120))
        );
        assert_eq!(orientation(&raw), Some(6));
    }

    /// IFD0 holding one entry, a DefaultCropSize LONG pair at offset 26, and padded raw sizes.
    fn sony_like(crop: (u32, u32)) -> Vec<u8> {
        let mut raw = vec![0u8; 34];
        raw[..8].copy_from_slice(&[b'I', b'I', 42, 0, 8, 0, 0, 0]);
        raw[8..10].copy_from_slice(&1u16.to_le_bytes());
        raw[10..22].copy_from_slice(&[0x20, 0xc6, 4, 0, 2, 0, 0, 0, 26, 0, 0, 0]);
        raw[26..30].copy_from_slice(&crop.0.to_le_bytes());
        raw[30..34].copy_from_slice(&crop.1.to_le_bytes());
        raw
    }

    #[test]
    fn p58_raw_frame_is_the_default_crop_size() {
        assert_eq!(raw_frame(&sony_like((6240, 4160))), Some((6240, 4160)));
        assert_eq!(raw_frame(&sony_like((6000, 4000))), Some((6000, 4000)));
        assert_eq!(raw_frame(b"II*\0\0\0\0\0"), None);
    }

    #[test]
    fn p58_original_matches_the_camera_frame() {
        let a7r_v = Some((6240, 4160));
        let frame = |w, h| Some(Frame { w, h });
        assert_eq!(
            compose_frame((6240, 4160), a7r_v, Some(1)),
            frame(6240, 4160)
        );
        assert_eq!(
            compose_frame((6240, 4160), a7r_v, Some(6)),
            frame(4160, 6240)
        );
        assert_eq!(
            compose_frame((6240, 4160), Some((9600, 6376)), None),
            frame(6240, 4160)
        );
        assert_eq!(
            compose_frame((1088, 1080), Some((6000, 4000)), Some(1)),
            None
        );
        assert_eq!(
            compose_frame((1500, 1000), Some((6000, 4000)), Some(1)),
            None
        );
        assert_eq!(
            compose_frame((1600, 1066), Some((6000, 4000)), Some(1)),
            frame(1600, 1066)
        );
        assert_eq!(
            compose_frame((6240, 4160), Some((6656, 4608)), Some(1)),
            None
        );
        assert_eq!(compose_frame((6240, 4160), None, Some(1)), None);
    }
}
