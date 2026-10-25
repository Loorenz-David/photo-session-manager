use psm_core::{
    Result,
    ports::raw_previews::{Extracted, RawPreviews},
};
use std::{collections::HashSet, fs, path::Path};

pub struct EmbeddedJpeg;
impl RawPreviews for EmbeddedJpeg {
    fn extract(&self, raw: &Path, dest: &Path) -> Result<Option<Extracted>> {
        let bytes = fs::read(raw)?;
        let Some((offset, len, _, _)) = largest_embedded_jpeg(&bytes) else {
            return Ok(None);
        };
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp = dest.with_extension("jpg.tmp");
        fs::write(&temp, &bytes[offset..offset + len])?;
        fs::rename(temp, dest)?;
        Ok(Some(Extracted {
            orientation: orientation(&bytes),
        }))
    }
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
}
