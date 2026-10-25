pub struct SyntheticRaw;

impl SyntheticRaw {
    pub fn jpeg(w: u16, h: u16, valid_trailer: bool) -> Vec<u8> {
        let mut out = vec![0xff, 0xd8, 0xff, 0xc0, 0, 11, 8];
        out.extend_from_slice(&h.to_be_bytes());
        out.extend_from_slice(&w.to_be_bytes());
        out.extend_from_slice(&[1, 1, 0x11, 0]);
        if valid_trailer {
            out.extend_from_slice(&[0xff, 0xd9]);
        }
        out
    }

    pub fn build(
        streams: &[(u16, u16, bool)],
        orientation: u8,
        exif_datetime: Option<&str>,
    ) -> (Vec<u8>, Vec<Vec<u8>>) {
        let jpegs: Vec<_> = streams
            .iter()
            .map(|(w, h, valid)| Self::jpeg(*w, *h, *valid))
            .collect();
        let count = streams.len().max(1);
        let first_tags =
            1 + usize::from(!streams.is_empty()) * 2 + usize::from(exif_datetime.is_some());
        let size = |tags: usize| 2 + tags * 12 + 4;
        let mut offsets = Vec::new();
        let mut cursor = 8;
        for i in 0..count {
            offsets.push(cursor);
            cursor += size(if i == 0 { first_tags } else { 2 });
        }
        let exif_offset = exif_datetime.map(|_| {
            let offset = cursor;
            cursor += size(1);
            offset
        });
        let date_offset = exif_datetime.map(|date| {
            let offset = cursor;
            cursor += date.len() + 1;
            offset
        });
        let mut bytes = vec![0u8; cursor];
        bytes[..8].copy_from_slice(&[b'I', b'I', 42, 0, 8, 0, 0, 0]);
        for (i, offset) in offsets.iter().enumerate() {
            let tags = if i == 0 { first_tags } else { 2 };
            bytes[*offset..*offset + 2].copy_from_slice(&(tags as u16).to_le_bytes());
            let mut slot = 0;
            if i == 0 {
                write_tag(&mut bytes, *offset, slot, 0x0112, 3, 1, orientation as u32);
                slot += 1;
                if let Some(exif) = exif_offset {
                    write_tag(&mut bytes, *offset, slot, 0x8769, 4, 1, exif as u32);
                    slot += 1;
                }
            }
            if let Some(jpeg) = jpegs.get(i) {
                write_tag(&mut bytes, *offset, slot, 0x0201, 4, 1, cursor as u32);
                write_tag(
                    &mut bytes,
                    *offset,
                    slot + 1,
                    0x0202,
                    4,
                    1,
                    jpeg.len() as u32,
                );
                bytes.extend_from_slice(jpeg);
                cursor += jpeg.len();
            }
            let next_at = *offset + 2 + tags * 12;
            bytes[next_at..next_at + 4]
                .copy_from_slice(&(offsets.get(i + 1).copied().unwrap_or(0) as u32).to_le_bytes());
        }
        if let (Some(exif), Some(date_at), Some(date)) = (exif_offset, date_offset, exif_datetime) {
            bytes[exif..exif + 2].copy_from_slice(&1u16.to_le_bytes());
            write_tag(
                &mut bytes,
                exif,
                0,
                0x9003,
                2,
                (date.len() + 1) as u32,
                date_at as u32,
            );
            bytes[date_at..date_at + date.len()].copy_from_slice(date.as_bytes());
        }
        (bytes, jpegs)
    }
}

fn write_tag(
    bytes: &mut [u8],
    ifd: usize,
    slot: usize,
    tag: u16,
    kind: u16,
    count: u32,
    value: u32,
) {
    let at = ifd + 2 + slot * 12;
    bytes[at..at + 2].copy_from_slice(&tag.to_le_bytes());
    bytes[at + 2..at + 4].copy_from_slice(&kind.to_le_bytes());
    bytes[at + 4..at + 8].copy_from_slice(&count.to_le_bytes());
    bytes[at + 8..at + 12].copy_from_slice(&value.to_le_bytes());
}
