use super::{InstalledSupport, manifest};
use psm_core::{Error, Result};
use psm_infra::paths::AppPaths;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

/// P47 S2: the copied helper needs this entitlement to load Sony's differently signed dylibs.
pub const SIDECAR_NEEDS_LIBRARY_VALIDATION_OFF: bool = true;
const ENTITLEMENTS: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><plist version=\"1.0\"><dict><key>com.apple.security.cs.disable-library-validation</key><true/></dict></plist>";

fn invalid(file: impl Into<String>) -> Error {
    Error::SupportArchiveInvalid { file: file.into() }
}
fn hash(path: &Path) -> Result<Vec<u8>> {
    Ok(Sha256::digest(fs::read(path)?).to_vec())
}
fn macho(path: &Path) -> Result<bool> {
    let mut file = fs::File::open(path)?;
    let mut magic = [0; 4];
    if file.read(&mut magic)? < 4 {
        return Ok(false);
    }
    Ok(matches!(
        magic,
        [0xfe, 0xed, 0xfa, 0xcf]
            | [0xfe, 0xed, 0xfa, 0xce]
            | [0xca, 0xfe, 0xba, 0xbe]
            | [0xcf, 0xfa, 0xed, 0xfe]
    ))
}
fn sign(path: &Path, entitlements: Option<&Path>) -> Result<()> {
    if !macho(path)? {
        return Ok(());
    }
    let mut cmd = Command::new("codesign");
    cmd.args(["-s", "-", "--force"]);
    if entitlements.is_some() {
        cmd.args(["--options", "runtime"]);
    }
    if let Some(plist) = entitlements {
        cmd.arg("--entitlements").arg(plist);
    }
    let output = cmd.arg(path).output()?;
    if !output.status.success() {
        return Err(Error::Provider("Camera support signing failed".into()));
    }
    Ok(())
}

fn version(program: &Path) -> Result<String> {
    let mut child = Command::new(program)
        .arg("version")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if let Some(status) = child.try_wait()? {
            if !status.success() {
                return Err(Error::SupportVersionUnsupported {
                    version: "unavailable".into(),
                });
            }
            let mut stdout = String::new();
            child.stdout.take().unwrap().read_to_string(&mut stdout)?;
            let value: serde_json::Value =
                serde_json::from_str(stdout.lines().next().unwrap_or("")).map_err(|_| {
                    Error::SupportVersionUnsupported {
                        version: "unavailable".into(),
                    }
                })?;
            let payload = if value.get("event").and_then(|v| v.as_str()) == Some("crlab.version") {
                &value
            } else {
                value.get("fields").unwrap_or(&value)
            };
            if payload.get("sdk_compiled_in").and_then(|v| v.as_bool()) == Some(false) {
                return Err(Error::SupportVersionUnsupported {
                    version: "unavailable".into(),
                });
            }
            return Ok(payload
                .get("sdk_version")
                .and_then(|v| v.as_str())
                .unwrap_or("unavailable")
                .into());
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Error::SupportVersionUnsupported {
                version: "unavailable".into(),
            });
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub fn install(paths: &AppPaths, archive: &Path, bundled: &Path) -> Result<InstalledSupport> {
    install_with_manifest(paths, archive, bundled, &super::manifest_for(paths))
}

/// The explicit manifest argument is a test seam. Production always passes the compiled manifest.
pub fn install_with_manifest(
    paths: &AppPaths,
    archive: &Path,
    bundled: &Path,
    spec: &manifest::Manifest,
) -> Result<InstalledSupport> {
    let archive_name = archive
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let file = fs::File::open(archive).map_err(|_| invalid(&archive_name))?;
    let mut zip = zip::ZipArchive::new(file).map_err(|_| invalid(&archive_name))?;
    let located = manifest::verify(&mut zip, spec).map_err(invalid)?;
    let camera_root = paths.root.join("camera");
    fs::create_dir_all(&camera_root)?;
    let temp = tempfile::Builder::new()
        .prefix(".install-")
        .tempdir_in(&camera_root)?;
    for (name, bytes) in &located.files {
        let target = temp.path().join(name);
        fs::create_dir_all(target.parent().unwrap())?;
        fs::write(&target, bytes)?;
        if let Some(adapter) = name.strip_prefix("CrAdapter/") {
            let framework = temp
                .path()
                .join("Contents/Frameworks/CrAdapter")
                .join(adapter);
            fs::create_dir_all(framework.parent().unwrap())?;
            fs::write(framework, bytes)?;
        }
    }
    let sidecar = temp.path().join("camera-sidecar");
    fs::copy(bundled, &sidecar)?;
    fs::set_permissions(&sidecar, fs::Permissions::from_mode(0o755))?;
    // The fake's version probe reads behaviour.json beside its copied executable. This file is
    // temporary and is removed before the support folder is installed.
    let fake_behaviour = bundled.parent().unwrap().join("rec/behaviour.json");
    if fake_behaviour.is_file() {
        fs::create_dir_all(temp.path().join("rec"))?;
        fs::copy(fake_behaviour, temp.path().join("rec/behaviour.json"))?;
    }
    let entitlements = temp.path().join("camera-sidecar.entitlements.plist");
    if SIDECAR_NEEDS_LIBRARY_VALIDATION_OFF {
        fs::write(&entitlements, ENTITLEMENTS)?;
    }
    for (name, _) in &located.files {
        sign(&temp.path().join(name), None)?;
        if let Some(adapter) = name.strip_prefix("CrAdapter/") {
            sign(
                &temp
                    .path()
                    .join("Contents/Frameworks/CrAdapter")
                    .join(adapter),
                None,
            )?;
        }
    }
    sign(
        &sidecar,
        SIDECAR_NEEDS_LIBRARY_VALIDATION_OFF.then_some(entitlements.as_path()),
    )?;
    let sdk = version(&sidecar)?;
    if !manifest::SUPPORTED_SDK_VERSIONS.contains(&sdk.as_str()) || sdk != spec.version {
        return Err(Error::SupportVersionUnsupported { version: sdk });
    }
    if temp.path().join("rec").exists() {
        fs::remove_dir_all(temp.path().join("rec"))?;
    }
    let root = camera_root.join("sony");
    fs::create_dir_all(&root)?;
    let folder = root.join(&sdk);
    let old = root.join(format!(".old-{}", uuid::Uuid::new_v4()));
    if folder.exists() {
        fs::rename(&folder, &old)?;
    }
    if let Err(error) = fs::rename(temp.path(), &folder) {
        if old.exists() {
            let _ = fs::rename(&old, &folder);
        }
        return Err(error.into());
    }
    if old.exists() {
        fs::remove_dir_all(old)?;
    }
    Ok(InstalledSupport {
        make: "sony",
        version: sdk,
        sidecar_path: folder.join("camera-sidecar"),
        folder,
    })
}

pub fn refresh_sidecar_copy(paths: &AppPaths, bundled: &Path) -> Result<()> {
    let Some(installed) = super::installed(paths) else {
        return Ok(());
    };
    if hash(&installed.sidecar_path)? == hash(bundled)? {
        return Ok(());
    }
    let temp = installed.folder.join("camera-sidecar.new");
    fs::copy(bundled, &temp)?;
    fs::set_permissions(&temp, fs::Permissions::from_mode(0o755))?;
    let entitlements = installed.folder.join("camera-sidecar.entitlements.plist");
    if SIDECAR_NEEDS_LIBRARY_VALIDATION_OFF {
        fs::write(&entitlements, ENTITLEMENTS)?;
    }
    sign(
        &temp,
        SIDECAR_NEEDS_LIBRARY_VALIDATION_OFF.then_some(entitlements.as_path()),
    )?;
    fs::rename(temp, installed.sidecar_path)?;
    Ok(())
}
