//! A Capture One master stand-in (10D MC-28): a 16-bit RGB, uncompressed, little-endian TIFF
//! written by hand (TIFF 6.0 / EXIF 2.3 layouts, one strip), with optional `Make`, `Model`,
//! `Copyright`, `Orientation`, an Exif sub-IFD carrying `DateTimeOriginal` and a GPS sub-IFD
//! carrying `GPSLatitude`, filled with a test pattern.

/// The pixels of a synthetic master (16-bit sRGB samples).
#[derive(Debug, Clone)]
pub enum Pattern {
    Solid([u16; 3]),
    /// 1-px lines of `ink` at the listed columns and rows on `paper`.
    Marks {
        cols: Vec<u32>,
        rows: Vec<u32>,
        ink: [u16; 3],
        paper: [u16; 3],
    },
    /// Grey `0x8000`, column `mid − 1` black, column `mid` white.
    ColumnMarker {
        mid: u32,
    },
    /// A one-pixel black/white checkerboard (black at `(0, 0)`).
    Checker1px,
}

/// The metadata a synthetic master carries; `None`/`false` writes nothing.
#[derive(Debug, Clone, Default)]
pub struct Meta {
    pub make: Option<String>,
    pub model: Option<String>,
    pub date_time_original: Option<String>,
    pub copyright: Option<String>,
    pub orientation: Option<u16>,
    pub gps: bool,
}

#[derive(Debug, Clone)]
pub struct SyntheticMaster {
    pub w: u32,
    pub h: u32,
    pub pattern: Pattern,
    pub meta: Meta,
}

pub const BLACK: [u16; 3] = [0; 3];
pub const WHITE: [u16; 3] = [0xFFFF; 3];
pub const PAPER: [u16; 3] = [0xC000; 3];

struct Entry {
    tag: u16,
    kind: u16,
    count: u32,
    data: Vec<u8>,
}
fn short(tag: u16, values: &[u16]) -> Entry {
    Entry {
        tag,
        kind: 3,
        count: values.len() as u32,
        data: values.iter().flat_map(|v| v.to_le_bytes()).collect(),
    }
}
fn long(tag: u16, value: u32) -> Entry {
    Entry {
        tag,
        kind: 4,
        count: 1,
        data: value.to_le_bytes().to_vec(),
    }
}
fn ascii(tag: u16, text: &str) -> Entry {
    let mut data = text.as_bytes().to_vec();
    data.push(0);
    Entry {
        tag,
        kind: 2,
        count: data.len() as u32,
        data,
    }
}
fn ifd_size(entries: &[Entry]) -> usize {
    let extra: usize = entries
        .iter()
        .filter(|e| e.data.len() > 4)
        .map(|e| e.data.len().div_ceil(2) * 2)
        .sum();
    2 + 12 * entries.len() + 4 + extra
}
fn write_ifd(out: &mut Vec<u8>, entries: &mut [Entry]) {
    entries.sort_by_key(|e| e.tag);
    let start = out.len();
    let mut extra_at = start + 2 + 12 * entries.len() + 4;
    let mut extra = Vec::new();
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    for e in entries.iter() {
        out.extend_from_slice(&e.tag.to_le_bytes());
        out.extend_from_slice(&e.kind.to_le_bytes());
        out.extend_from_slice(&e.count.to_le_bytes());
        if e.data.len() <= 4 {
            let mut inline = e.data.clone();
            inline.resize(4, 0);
            out.extend_from_slice(&inline);
        } else {
            out.extend_from_slice(&(extra_at as u32).to_le_bytes());
            let mut data = e.data.clone();
            if data.len() % 2 == 1 {
                data.push(0);
            }
            extra_at += data.len();
            extra.extend_from_slice(&data);
        }
    }
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&extra);
}

impl SyntheticMaster {
    pub fn solid(w: u32, h: u32, colour: [u16; 3]) -> Self {
        Self {
            w,
            h,
            pattern: Pattern::Solid(colour),
            meta: Meta::default(),
        }
    }
    pub fn with(w: u32, h: u32, pattern: Pattern) -> Self {
        Self {
            w,
            h,
            pattern,
            meta: Meta::default(),
        }
    }
    fn pixel(&self, x: u32, y: u32) -> [u16; 3] {
        match &self.pattern {
            Pattern::Solid(c) => *c,
            Pattern::Marks {
                cols,
                rows,
                ink,
                paper,
            } => {
                if cols.contains(&x) || rows.contains(&y) {
                    *ink
                } else {
                    *paper
                }
            }
            Pattern::ColumnMarker { mid } => {
                if x + 1 == *mid {
                    BLACK
                } else if x == *mid {
                    WHITE
                } else {
                    [0x8000; 3]
                }
            }
            Pattern::Checker1px => {
                if (x + y).is_multiple_of(2) {
                    BLACK
                } else {
                    WHITE
                }
            }
        }
    }
    /// The TIFF file's bytes.
    pub fn bytes(&self) -> Vec<u8> {
        let strip_len = self.w as usize * self.h as usize * 6;
        let mut ifd0 = vec![
            long(0x0100, self.w),
            long(0x0101, self.h),
            short(0x0102, &[16, 16, 16]),
            short(0x0103, &[1]),
            short(0x0106, &[2]),
            long(0x0111, 0),
            short(0x0115, &[3]),
            long(0x0116, self.h),
            long(0x0117, strip_len as u32),
            short(0x011C, &[1]),
        ];
        if let Some(make) = &self.meta.make {
            ifd0.push(ascii(0x010F, make));
        }
        if let Some(model) = &self.meta.model {
            ifd0.push(ascii(0x0110, model));
        }
        if let Some(orientation) = self.meta.orientation {
            ifd0.push(short(0x0112, &[orientation]));
        }
        if let Some(copyright) = &self.meta.copyright {
            ifd0.push(ascii(0x8298, copyright));
        }
        let mut exif = Vec::new();
        if let Some(date) = &self.meta.date_time_original {
            exif.push(ascii(0x9003, date));
            ifd0.push(long(0x8769, 0));
        }
        let mut gps = Vec::new();
        if self.meta.gps {
            gps.push(Entry {
                tag: 0x0000,
                kind: 1,
                count: 4,
                data: vec![2, 3, 0, 0],
            });
            gps.push(ascii(0x0001, "N"));
            gps.push(Entry {
                tag: 0x0002,
                kind: 5,
                count: 3,
                data: [59u32, 1, 19, 1, 48, 1]
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect(),
            });
            ifd0.push(long(0x8825, 0));
        }
        let exif_at = 8 + ifd_size(&ifd0);
        let gps_at = exif_at + if exif.is_empty() { 0 } else { ifd_size(&exif) };
        let strip_at = gps_at + if gps.is_empty() { 0 } else { ifd_size(&gps) };
        for e in &mut ifd0 {
            match e.tag {
                0x0111 => e.data = (strip_at as u32).to_le_bytes().to_vec(),
                0x8769 => e.data = (exif_at as u32).to_le_bytes().to_vec(),
                0x8825 => e.data = (gps_at as u32).to_le_bytes().to_vec(),
                _ => {}
            }
        }
        let mut out = b"II".to_vec();
        out.extend_from_slice(&42u16.to_le_bytes());
        out.extend_from_slice(&8u32.to_le_bytes());
        write_ifd(&mut out, &mut ifd0);
        if !exif.is_empty() {
            write_ifd(&mut out, &mut exif);
        }
        if !gps.is_empty() {
            write_ifd(&mut out, &mut gps);
        }
        assert_eq!(out.len(), strip_at, "synthetic master layout");
        out.reserve(strip_len);
        for y in 0..self.h {
            for x in 0..self.w {
                for v in self.pixel(x, y) {
                    out.extend_from_slice(&v.to_le_bytes());
                }
            }
        }
        out
    }
}
