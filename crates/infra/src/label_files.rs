//! `DiskLabelFiles` (book 09C §6.1, §6.4; `master_plan_labels.md` §4 Files, §5 MC-L2.5, .13): the
//! label preview pictures (a PNG of the label PDF's first page, made with macOS's `sips`, under
//! `<previews>/labels/`, which the webview may read) and the saved label PDFs (the labels folder,
//! `~/Documents/Session Manager Labels`, given by `App::build` from `AppPaths.home`), opened in
//! Preview.

use psm_core::{
    Error, Result,
    mechanisms::labels::{PREVIEW_EDGE, label_file_name, numbered_name},
    ports::labels::LabelFiles,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const SIPS: &str = "/usr/bin/sips";
const OPEN: &str = "/usr/bin/open";

pub struct DiskLabelFiles {
    previews_root: PathBuf,
    labels_dir: PathBuf,
}

impl DiskLabelFiles {
    /// `previews_root` = `<data>/previews`; `labels_dir` = the labels folder.
    pub fn new(previews_root: PathBuf, labels_dir: PathBuf) -> Self {
        Self {
            previews_root,
            labels_dir,
        }
    }

    /// `<previews>/labels/`, where preview pictures live.
    pub fn previews_dir(&self) -> PathBuf {
        self.previews_root.join("labels")
    }

    /// The labels folder.
    pub fn labels_dir(&self) -> &Path {
        &self.labels_dir
    }

    fn run_sips(source: &Path, out: &Path) -> Result<()> {
        let status = Command::new(SIPS)
            .args(["-s", "format", "png", "-Z", &PREVIEW_EDGE.to_string()])
            .arg(source)
            .arg("--out")
            .arg(out)
            .output()?;
        if !status.status.success() {
            return Err(Error::Provider(format!(
                "sips exited with {}",
                status.status.code().unwrap_or(-1)
            )));
        }
        if !out.is_file() {
            return Err(Error::Provider("sips wrote no picture".into()));
        }
        Ok(())
    }

    /// The first free name for `name` in the folder: `name`, else `<stem> (2).pdf`, (3), ….
    fn free_name(&self, name: &str) -> PathBuf {
        let taken = |p: &Path| fs::symlink_metadata(p).is_ok();
        let first = self.labels_dir.join(name);
        if !taken(&first) {
            return first;
        }
        let mut n = 2;
        loop {
            let candidate = self.labels_dir.join(numbered_name(name, n));
            if !taken(&candidate) {
                return candidate;
            }
            n += 1;
        }
    }
}

impl LabelFiles for DiskLabelFiles {
    /// The PDF is written to `<previews>/labels/<name>.src.pdf` (never the system's temporary
    /// folder), `sips` renders it to `<name>.png`, and the temporary PDF is removed on every way
    /// out (projection r0, D1).
    fn raster(&self, pdf: &[u8], name: &str) -> Result<PathBuf> {
        let dir = self.previews_dir();
        fs::create_dir_all(&dir)?;
        let source = dir.join(format!("{name}.src.pdf"));
        let out = dir.join(format!("{name}.png"));
        let _ = fs::remove_file(&out);
        let result = fs::write(&source, pdf)
            .map_err(Error::from)
            .and_then(|()| Self::run_sips(&source, &out));
        let _ = fs::remove_file(&source);
        if result.is_err() {
            let _ = fs::remove_file(&out);
        }
        result.map(|()| out)
    }

    fn clear_previews(&self) -> Result<()> {
        match fs::remove_dir_all(self.previews_dir()) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// MC-L2.13: the folder is created when missing; the name is `label_file_name(filename)`,
    /// numbered while taken; the bytes go to `<name>.part` and are renamed (the `.part` is
    /// removed on a failure).
    fn save(&self, filename: &str, bytes: &[u8]) -> Result<PathBuf> {
        fs::create_dir_all(&self.labels_dir)?;
        let target = self.free_name(&label_file_name(filename));
        let mut part = target.clone().into_os_string();
        part.push(".part");
        let part = PathBuf::from(part);
        let result = fs::write(&part, bytes).and_then(|()| fs::rename(&part, &target));
        if let Err(error) = result {
            let _ = fs::remove_file(&part);
            return Err(error.into());
        }
        Ok(target)
    }

    /// Never run by a test (it would start Preview on the dev Mac).
    fn open(&self, path: &Path) -> Result<()> {
        let status = Command::new(OPEN)
            .args(["-a", "Preview"])
            .arg(path)
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::Provider(format!(
                "open exited with {}",
                status.code().unwrap_or(-1)
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files() -> (tempfile::TempDir, DiskLabelFiles) {
        let dir = tempfile::tempdir().unwrap();
        let files = DiskLabelFiles::new(
            dir.path().join("previews"),
            dir.path().join("home/Documents/Session Manager Labels"),
        );
        (dir, files)
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    /// A one-page PDF of 70 × 100 mm (MediaBox 198.43 × 283.46 pt) with one line of text.
    fn one_page_pdf() -> Vec<u8> {
        let content = "BT /F1 12 Tf 20 200 Td (Test label) Tj ET";
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 198.43 283.46] /Contents 4 0 R \
             /Resources << /Font << /F1 5 0 R >> >> >>"
                .to_owned(),
            format!(
                "<< /Length {} >>\nstream\n{content}\nendstream",
                content.len()
            ),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
        ];
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::new();
        for (i, body) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.extend(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
        }
        let xref = pdf.len();
        pdf.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
        for offset in offsets {
            pdf.extend(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                objects.len() + 1
            )
            .as_bytes(),
        );
        pdf
    }

    #[test]
    fn l3_c4a_save_creates_the_folder_and_leaves_no_part_file() {
        let (_dir, files) = files();
        let path = files.save("labels-2026-11-20.pdf", b"%PDF-1 a").unwrap();
        assert_eq!(path, files.labels_dir().join("labels-2026-11-20.pdf"));
        assert_eq!(fs::read(&path).unwrap(), b"%PDF-1 a");
        assert_eq!(names(files.labels_dir()), vec!["labels-2026-11-20.pdf"]);
    }

    #[test]
    fn l3_c4b_a_taken_name_is_numbered() {
        let (_dir, files) = files();
        files.save("labels-2026-11-20.pdf", b"first").unwrap();
        let second = files.save("labels-2026-11-20.pdf", b"second").unwrap();
        let third = files.save("labels-2026-11-20.pdf", b"third").unwrap();
        assert_eq!(second, files.labels_dir().join("labels-2026-11-20 (2).pdf"));
        assert_eq!(third, files.labels_dir().join("labels-2026-11-20 (3).pdf"));
        assert_eq!(fs::read(&second).unwrap(), b"second");
        assert_eq!(fs::read(&third).unwrap(), b"third");
        assert_eq!(
            fs::read(files.labels_dir().join("labels-2026-11-20.pdf")).unwrap(),
            b"first"
        );
        assert_eq!(
            names(files.labels_dir()),
            vec![
                "labels-2026-11-20 (2).pdf",
                "labels-2026-11-20 (3).pdf",
                "labels-2026-11-20.pdf"
            ]
        );
    }

    #[test]
    fn l3_c4c_raster_writes_a_png_and_keeps_no_temporary_pdf() {
        let (_dir, files) = files();
        let png = files.raster(&one_page_pdf(), "x").unwrap();
        assert_eq!(png, files.previews_dir().join("x.png"));
        let bytes = fs::read(&png).unwrap();
        assert_eq!(
            &bytes[..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );
        assert!(files.raster(b"not a pdf", "y").is_err());
        assert!(!files.previews_dir().join("y.png").exists());
        assert_eq!(names(&files.previews_dir()), vec!["x.png"]);
    }

    #[test]
    fn l3_c4d_nothing_is_written_outside_the_folder() {
        let (_dir, files) = files();
        let evil = files.save("../../evil.pdf", b"e").unwrap();
        let empty = files.save("", b"l").unwrap();
        assert_eq!(evil, files.labels_dir().join("evil.pdf"));
        assert_eq!(empty, files.labels_dir().join("labels.pdf"));
        let parent = files.labels_dir().parent().unwrap();
        assert!(!parent.join("evil.pdf").exists());
        assert!(!parent.parent().unwrap().join("evil.pdf").exists());
    }

    #[test]
    fn l3_c4e_clear_previews_removes_only_the_label_pictures() {
        let (_dir, files) = files();
        fs::create_dir_all(files.previews_dir()).unwrap();
        fs::write(files.previews_dir().join("a-1.png"), b"1").unwrap();
        fs::write(files.previews_dir().join("a-2.png"), b"2").unwrap();
        let other = files.previews_dir().parent().unwrap().join("other.jpg");
        fs::write(&other, b"o").unwrap();
        files.clear_previews().unwrap();
        assert!(!files.previews_dir().join("a-1.png").exists());
        assert!(!files.previews_dir().join("a-2.png").exists());
        assert_eq!(fs::read(&other).unwrap(), b"o");
    }

    #[test]
    fn l3_c4f_a_folder_that_is_a_file_fails_without_a_part_file() {
        let (_dir, files) = files();
        let folder = files.labels_dir().to_path_buf();
        fs::create_dir_all(folder.parent().unwrap()).unwrap();
        fs::write(&folder, b"a regular file").unwrap();
        assert!(files.save("a.pdf", b"bytes").is_err());
        assert_eq!(
            names(folder.parent().unwrap()),
            vec!["Session Manager Labels"]
        );
        assert_eq!(fs::read(&folder).unwrap(), b"a regular file");
    }
}
