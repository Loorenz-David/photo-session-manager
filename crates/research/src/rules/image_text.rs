//! The parts of an exported file the export's secret check reads (MC-A11; owner corrections A7,
//! A8). Listing images are stored as the bytes the shop's server sent, under a `.jpg` name, so
//! they are JPEG, PNG, WebP, GIF or AVIF/HEIF. Their coded pixel data holds no text and a short
//! stored value turns up in it by chance, which stopped real exports. A recognised image is read
//! only where text can live:
//!
//! - JPEG: the header, SOI up to the first start-of-scan marker (EXIF, XMP, ICC, comments).
//! - PNG: the text chunks `tEXt`, `iTXt`, `zTXt` (inflated when compressed) and `eXIf`, plus any
//!   bytes after `IEND`.
//! - WebP: the `EXIF` and `XMP ` chunks.
//! - GIF: the comment, plain-text and application extensions (XMP lives in the latter).
//! - AVIF/HEIF: every top-level box except `mdat`. Metadata items of these formats may sit inside
//!   `mdat` and are not read: the app never writes into downloaded bytes, and what the check
//!   guards is the record the app wrote.
//!
//! Anything not recognised, or an image whose structure cannot be walked to its end, is read
//! whole (fail closed).

use std::{borrow::Cow, io::Read};

/// Inflated text larger than this is treated as unwalkable (the file is read whole).
const INFLATE_LIMIT: u64 = 4 * 1024 * 1024;

/// The parts of `bytes` the secret check reads.
pub fn scanned_parts(bytes: &[u8]) -> Vec<Cow<'_, [u8]>> {
    let parts = if bytes.starts_with(&[0xFF, 0xD8]) {
        jpeg_header_len(bytes).map(|len| vec![Cow::Borrowed(&bytes[..len])])
    } else if bytes.starts_with(PNG_SIGNATURE) {
        png_text(bytes)
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        webp_text(bytes)
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        gif_text(bytes)
    } else if is_heif(bytes) {
        heif_boxes(bytes)
    } else {
        None
    };
    parts.unwrap_or_else(|| vec![Cow::Borrowed(bytes)])
}

/// The length of a JPEG's header (SOI up to, not including, the first SOS marker); `None` when
/// its segments run past the end.
fn jpeg_header_len(bytes: &[u8]) -> Option<usize> {
    let mut i = 2;
    loop {
        if *bytes.get(i)? != 0xFF {
            return None;
        }
        // Fill bytes before a marker.
        while *bytes.get(i + 1)? == 0xFF {
            i += 1;
        }
        let marker = *bytes.get(i + 1)?;
        match marker {
            0xDA => return Some(i),
            0xD9 => return None,
            0x01 | 0xD0..=0xD7 => i += 2,
            _ => {
                let len = usize::from(u16::from_be_bytes([*bytes.get(i + 2)?, *bytes.get(i + 3)?]));
                if len < 2 {
                    return None;
                }
                i += 2 + len;
            }
        }
    }
}

const PNG_SIGNATURE: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

fn inflate(data: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    flate2::read::ZlibDecoder::new(data)
        .take(INFLATE_LIMIT + 1)
        .read_to_end(&mut out)
        .ok()?;
    (out.len() as u64 <= INFLATE_LIMIT).then_some(out)
}

fn be_u32(bytes: &[u8], at: usize) -> Option<usize> {
    let b = bytes.get(at..at + 4)?;
    usize::try_from(u32::from_be_bytes([b[0], b[1], b[2], b[3]])).ok()
}

fn le_u32(bytes: &[u8], at: usize) -> Option<usize> {
    let b = bytes.get(at..at + 4)?;
    usize::try_from(u32::from_le_bytes([b[0], b[1], b[2], b[3]])).ok()
}

/// PNG: chunk = length (BE u32), type, data, CRC.
fn png_text(bytes: &[u8]) -> Option<Vec<Cow<'_, [u8]>>> {
    let mut parts = Vec::new();
    let mut i = PNG_SIGNATURE.len();
    loop {
        let len = be_u32(bytes, i)?;
        let kind = bytes.get(i + 4..i + 8)?;
        let start = i + 8;
        let end = start.checked_add(len)?;
        let data = bytes.get(start..end)?;
        let next = end.checked_add(4)?;
        if next > bytes.len() {
            return None;
        }
        match kind {
            b"tEXt" | b"eXIf" => parts.push(Cow::Borrowed(data)),
            b"zTXt" => {
                // keyword \0 method compressed-text
                let nul = data.iter().position(|b| *b == 0)?;
                parts.push(Cow::Borrowed(&data[..nul]));
                parts.push(Cow::Owned(inflate(data.get(nul + 2..)?)?));
            }
            b"iTXt" => {
                // keyword \0 flag method language \0 translated keyword \0 text
                let nul = data.iter().position(|b| *b == 0)?;
                let flag = *data.get(nul + 1)?;
                let rest = data.get(nul + 3..)?;
                let lang = rest.iter().position(|b| *b == 0)?;
                let after_lang = &rest[lang + 1..];
                let translated = after_lang.iter().position(|b| *b == 0)?;
                let text = &after_lang[translated + 1..];
                let head_len = data.len() - text.len();
                parts.push(Cow::Borrowed(&data[..head_len]));
                if flag == 0 {
                    parts.push(Cow::Borrowed(text));
                } else {
                    parts.push(Cow::Owned(inflate(text)?));
                }
            }
            b"IEND" => {
                if next < bytes.len() {
                    parts.push(Cow::Borrowed(&bytes[next..]));
                }
                return Some(parts);
            }
            _ => {}
        }
        i = next;
    }
}

/// WebP: RIFF header, then chunk = FourCC, size (LE u32), data, a pad byte when the size is odd.
fn webp_text(bytes: &[u8]) -> Option<Vec<Cow<'_, [u8]>>> {
    let mut parts = Vec::new();
    let mut i = 12;
    while i < bytes.len() {
        let kind = bytes.get(i..i + 4)?;
        let len = le_u32(bytes, i + 4)?;
        let start = i + 8;
        let end = start.checked_add(len)?;
        let data = bytes.get(start..end)?;
        if kind == b"EXIF" || kind == b"XMP " {
            parts.push(Cow::Borrowed(data));
        }
        i = end + (len & 1);
    }
    Some(parts)
}

/// GIF: the data of an extension's sub-blocks (size byte, data) up to the zero terminator, and
/// the index after it.
fn gif_sub_blocks(bytes: &[u8], mut i: usize) -> Option<(Vec<u8>, usize)> {
    let mut data = Vec::new();
    loop {
        let size = usize::from(*bytes.get(i)?);
        i += 1;
        if size == 0 {
            return Some((data, i));
        }
        data.extend_from_slice(bytes.get(i..i + size)?);
        i += size;
    }
}

/// The size of a GIF colour table announced by a packed flags byte.
fn gif_table(flags: u8) -> usize {
    if flags & 0x80 == 0 {
        0
    } else {
        3 * (1 << ((flags & 0x07) + 1))
    }
}

/// GIF: header, logical screen descriptor, global table, then extensions and images up to the
/// trailer.
fn gif_text(bytes: &[u8]) -> Option<Vec<Cow<'_, [u8]>>> {
    let mut parts = Vec::new();
    let mut i = 13 + gif_table(*bytes.get(10)?);
    loop {
        match *bytes.get(i)? {
            0x21 => {
                let label = *bytes.get(i + 1)?;
                let (data, next) = gif_sub_blocks(bytes, i + 2)?;
                // Plain text, comment and application extensions; not the graphic control.
                if matches!(label, 0x01 | 0xFE | 0xFF) {
                    parts.push(Cow::Owned(data));
                }
                i = next;
            }
            0x2C => {
                let flags = *bytes.get(i + 9)?;
                // Descriptor (10 bytes), local table, LZW minimum code size, image sub-blocks.
                let (_, next) = gif_sub_blocks(bytes, i + 10 + gif_table(flags) + 1)?;
                i = next;
            }
            0x3B => {
                if i + 1 < bytes.len() {
                    parts.push(Cow::Borrowed(&bytes[i + 1..]));
                }
                return Some(parts);
            }
            _ => return None,
        }
    }
}

/// An ISO-BMFF file whose `ftyp` major brand is an AVIF or HEIF one.
fn is_heif(bytes: &[u8]) -> bool {
    bytes.get(4..8) == Some(b"ftyp".as_slice())
        && matches!(
            bytes.get(8..12),
            Some(
                b"avif"
                    | b"avis"
                    | b"heic"
                    | b"heix"
                    | b"heim"
                    | b"heis"
                    | b"hevc"
                    | b"hevx"
                    | b"mif1"
                    | b"msf1"
            )
        )
}

/// AVIF/HEIF: every top-level box except `mdat`.
fn heif_boxes(bytes: &[u8]) -> Option<Vec<Cow<'_, [u8]>>> {
    let mut parts = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let size = be_u32(bytes, i)?;
        let kind = bytes.get(i + 4..i + 8)?;
        let end = match size {
            0 => bytes.len(),
            1 => {
                let b = bytes.get(i + 8..i + 16)?;
                let large = u64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                i.checked_add(usize::try_from(large).ok()?)?
            }
            n if n < 8 => return None,
            n => i.checked_add(n)?,
        };
        let whole = bytes.get(i..end)?;
        if kind != b"mdat" {
            parts.push(Cow::Borrowed(whole));
        }
        i = end;
    }
    Some(parts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const SECRET: &[u8] = b"Zq9-secret-Zq9";

    fn holds_secret(bytes: &[u8]) -> bool {
        scanned_parts(bytes)
            .iter()
            .any(|part| part.windows(SECRET.len()).any(|w| w == SECRET))
    }

    fn pixels() -> Vec<u8> {
        [b"\x12\x34".as_slice(), SECRET, b"\x56\x78"].concat()
    }

    fn deflate(data: &[u8]) -> Vec<u8> {
        let mut e = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        e.write_all(data).unwrap();
        e.finish().unwrap()
    }

    // JPEG ---------------------------------------------------------------------------------

    fn jpeg(comment: Option<&[u8]>, pixels: &[u8]) -> Vec<u8> {
        let mut out = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        out.extend_from_slice(b"JFIF\0\x01\x01\0\0\x01\0\x01\0\0");
        if let Some(text) = comment {
            out.extend_from_slice(&[0xFF, 0xFE]);
            out.extend_from_slice(&u16::try_from(text.len() + 2).unwrap().to_be_bytes());
            out.extend_from_slice(text);
        }
        out.extend_from_slice(&[0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3F, 0x00]);
        out.extend_from_slice(pixels);
        out.extend_from_slice(&[0xFF, 0xD9]);
        out
    }

    #[test]
    fn a8_jpeg_pixels_skipped_comment_read() {
        assert!(!holds_secret(&jpeg(None, &pixels())));
        assert!(holds_secret(&jpeg(Some(SECRET), b"\x01")));
        // Segments running past the end: read whole.
        assert!(holds_secret(
            &[&[0xFF, 0xD8, 0xFF, 0xE1, 0x7F, 0xFF][..], SECRET].concat()
        ));
    }

    // PNG ----------------------------------------------------------------------------------

    fn chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut out = u32::try_from(data.len()).unwrap().to_be_bytes().to_vec();
        out.extend_from_slice(kind);
        out.extend_from_slice(data);
        out.extend_from_slice(&[0, 0, 0, 0]); // CRC, not checked
        out
    }

    fn png(chunks: &[Vec<u8>]) -> Vec<u8> {
        let mut out = PNG_SIGNATURE.to_vec();
        out.extend(chunk(b"IHDR", &[0; 13]));
        for c in chunks {
            out.extend_from_slice(c);
        }
        out.extend(chunk(b"IEND", &[]));
        out
    }

    #[test]
    fn a8_png_pixels_skipped_text_chunks_read() {
        assert!(!holds_secret(&png(&[chunk(b"IDAT", &pixels())])));
        assert!(!holds_secret(&png(&[chunk(b"PLTE", &pixels())])));
        let text = [b"Comment\0".as_slice(), SECRET].concat();
        assert!(holds_secret(&png(&[chunk(b"tEXt", &text)])));
        let z = [b"Comment\0\0".as_slice(), &deflate(SECRET)].concat();
        assert!(holds_secret(&png(&[chunk(b"zTXt", &z)])));
        let i_plain = [b"XML:com.adobe.xmp\0\0\0en\0\0".as_slice(), SECRET].concat();
        assert!(holds_secret(&png(&[chunk(b"iTXt", &i_plain)])));
        let i_z = [
            b"XML:com.adobe.xmp\0\x01\0en\0\0".as_slice(),
            &deflate(SECRET),
        ]
        .concat();
        assert!(holds_secret(&png(&[chunk(b"iTXt", &i_z)])));
        assert!(holds_secret(&png(&[chunk(b"eXIf", SECRET)])));
        // Bytes after IEND are read.
        assert!(holds_secret(&[png(&[]), SECRET.to_vec()].concat()));
        // A chunk running past the end: read whole.
        let mut cut = png(&[chunk(b"IDAT", &pixels())]);
        cut.truncate(cut.len() - 12 - 4 - 1);
        assert!(holds_secret(&cut));
    }

    // WebP ---------------------------------------------------------------------------------

    fn riff(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut out = kind.to_vec();
        out.extend_from_slice(&u32::try_from(data.len()).unwrap().to_le_bytes());
        out.extend_from_slice(data);
        if data.len() % 2 == 1 {
            out.push(0);
        }
        out
    }

    fn webp(chunks: &[Vec<u8>]) -> Vec<u8> {
        let body: Vec<u8> = chunks.concat();
        let mut out = b"RIFF".to_vec();
        out.extend_from_slice(&u32::try_from(body.len() + 4).unwrap().to_le_bytes());
        out.extend_from_slice(b"WEBP");
        out.extend(body);
        out
    }

    #[test]
    fn a8_webp_pixels_skipped_exif_and_xmp_read() {
        let mut odd = pixels();
        odd.push(1);
        assert!(!holds_secret(&webp(&[
            riff(b"VP8 ", &odd),
            riff(b"ALPH", &pixels())
        ])));
        assert!(holds_secret(&webp(&[
            riff(b"VP8X", &[0; 10]),
            riff(b"VP8L", &odd),
            riff(b"EXIF", SECRET)
        ])));
        assert!(holds_secret(&webp(&[
            riff(b"VP8 ", &pixels()),
            riff(b"XMP ", SECRET)
        ])));
        // A chunk running past the end: read whole.
        let mut cut = webp(&[riff(b"VP8 ", &[pixels(), vec![0; 4]].concat())]);
        cut.truncate(cut.len() - 3);
        assert!(holds_secret(&cut));
    }

    // GIF ----------------------------------------------------------------------------------

    fn sub_blocks(data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        for piece in data.chunks(5) {
            out.push(u8::try_from(piece.len()).unwrap());
            out.extend_from_slice(piece);
        }
        out.push(0);
        out
    }

    fn gif(blocks: &[Vec<u8>]) -> Vec<u8> {
        // Header, 1x1 screen with a 2-colour global table.
        let mut out = b"GIF89a\x01\x00\x01\x00\x80\x00\x00".to_vec();
        out.extend_from_slice(&[0, 0, 0, 255, 255, 255]);
        for b in blocks {
            out.extend_from_slice(b);
        }
        out.push(0x3B);
        out
    }

    fn image(data: &[u8]) -> Vec<u8> {
        let mut out = vec![0x2C, 0, 0, 0, 0, 1, 0, 1, 0, 0x00, 0x02];
        out.extend(sub_blocks(data));
        out
    }

    fn extension(label: u8, data: &[u8]) -> Vec<u8> {
        [vec![0x21, label], sub_blocks(data)].concat()
    }

    #[test]
    fn a8_gif_pixels_skipped_comment_and_application_read() {
        assert!(!holds_secret(&gif(&[
            extension(0xF9, &[0, 0, 0, 0]),
            image(&pixels())
        ])));
        // A secret split across sub-blocks is still found.
        assert!(holds_secret(&gif(&[
            extension(0xFE, SECRET),
            image(b"\x01")
        ])));
        let xmp = [b"XMP DataXMP".as_slice(), SECRET].concat();
        assert!(holds_secret(&gif(&[extension(0xFF, &xmp), image(b"\x01")])));
        assert!(holds_secret(&gif(&[extension(0x01, SECRET)])));
        // An unknown block: read whole.
        let odd = gif(&[image(b"\x01"), [&[0x99][..], SECRET].concat()]);
        assert!(holds_secret(&odd));
    }

    // AVIF / HEIF --------------------------------------------------------------------------

    fn bmff(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut out = u32::try_from(data.len() + 8)
            .unwrap()
            .to_be_bytes()
            .to_vec();
        out.extend_from_slice(kind);
        out.extend_from_slice(data);
        out
    }

    #[test]
    fn a8_avif_mdat_skipped_other_boxes_read() {
        let ftyp = bmff(b"ftyp", b"avif\0\0\0\0mif1avif");
        assert!(!holds_secret(
            &[
                ftyp.clone(),
                bmff(b"meta", &[0; 12]),
                bmff(b"mdat", &pixels())
            ]
            .concat()
        ));
        assert!(holds_secret(
            &[ftyp.clone(), bmff(b"meta", SECRET), bmff(b"mdat", b"\x01")].concat()
        ));
        // A box running past the end: read whole.
        let mut cut = [ftyp, bmff(b"mdat", &pixels())].concat();
        cut.truncate(cut.len() - 2);
        assert!(holds_secret(&cut));
        // An ISO-BMFF file of another brand (a video): read whole.
        let mp4 = [bmff(b"ftyp", b"isom\0\0\0\0"), bmff(b"mdat", &pixels())].concat();
        assert!(holds_secret(&mp4));
    }

    #[test]
    fn a8_anything_else_is_read_whole() {
        assert!(holds_secret(
            &[b"{\"note\":\"".as_slice(), SECRET, b"\"}"].concat()
        ));
        assert!(holds_secret(&[b"BM".as_slice(), &pixels()].concat()));
    }
}
