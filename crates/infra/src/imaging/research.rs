//! MC-A5: reuses the composition renderer on an adjusted JPEG preview, then removes all JPEG
//! metadata segments without a second encode. Only the reduced copy remains at the destination.

use std::{fs, path::Path};

use psm_core::{
    Error, Result,
    domain::{
        composition::{CompositionValues, OutputSlot},
        output::{OutputFormat, OutputInfo},
    },
};

use super::render;

fn strip_metadata(jpeg: &[u8]) -> Result<Vec<u8>> {
    if !jpeg.starts_with(&[0xff, 0xd8]) {
        return Err(Error::InvalidData("research source is not JPEG".into()));
    }
    let mut out = vec![0xff, 0xd8];
    let mut at = 2;
    while at < jpeg.len() {
        if jpeg[at] != 0xff || at + 4 > jpeg.len() {
            return Err(Error::InvalidData("malformed JPEG segment".into()));
        }
        let start = at;
        while at < jpeg.len() && jpeg[at] == 0xff {
            at += 1;
        }
        let marker = *jpeg
            .get(at)
            .ok_or_else(|| Error::InvalidData("truncated JPEG".into()))?;
        at += 1;
        if marker == 0xda {
            out.extend_from_slice(&jpeg[start..]);
            return Ok(out);
        }
        if marker == 0xd9 {
            out.extend_from_slice(&jpeg[start..at]);
            return Ok(out);
        }
        let length = usize::from(u16::from_be_bytes([
            *jpeg
                .get(at)
                .ok_or_else(|| Error::InvalidData("truncated JPEG".into()))?,
            *jpeg
                .get(at + 1)
                .ok_or_else(|| Error::InvalidData("truncated JPEG".into()))?,
        ]));
        if length < 2 || at + length > jpeg.len() {
            return Err(Error::InvalidData("invalid JPEG segment length".into()));
        }
        at += length;
        if !(0xe0..=0xef).contains(&marker) && marker != 0xfe {
            out.extend_from_slice(&jpeg[start..at]);
        }
    }
    Err(Error::InvalidData("JPEG has no image data".into()))
}

pub fn render_research(
    preview: &Path,
    values: &CompositionValues,
    long_edge: u32,
    dest: &Path,
) -> Result<OutputInfo> {
    if long_edge == 0 {
        return Err(Error::InvalidValue(
            "research long edge must be positive".into(),
        ));
    }
    let bytes = fs::read(preview)?;
    if image::guess_format(&bytes).ok() != Some(image::ImageFormat::Jpeg) {
        return Err(Error::InvalidData(
            "research source is not an adjusted JPEG preview".into(),
        ));
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let nonce = uuid::Uuid::new_v4();
    let scratch = dest.with_extension(format!("research-{nonce}.jpg"));
    let output = render::render_output(
        preview,
        values,
        OutputSlot::Single,
        &OutputFormat {
            long_edge,
            quality: 85,
        },
        &scratch,
    );
    let result = (|| {
        let info = output?;
        let encoded = fs::read(&scratch)?;
        let stripped = strip_metadata(&encoded)?;
        let temp = dest.with_extension(format!("research-{nonce}.tmp"));
        let write = fs::write(&temp, stripped).and_then(|()| fs::rename(&temp, dest));
        if write.is_err() {
            let _ = fs::remove_file(&temp);
        }
        write?;
        Ok(info)
    })();
    let _ = fs::remove_file(&scratch);
    result
}
