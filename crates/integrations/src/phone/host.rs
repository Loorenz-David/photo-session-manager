//! The Mac's name and address for the phone (`master_plan_live.md` §6: the QR encodes the `.local`
//! URL — `scutil --get LocalHostName` + `.local`; the address URL uses the first private IPv4 of
//! `psm_infra::net::local_addresses`, shown with its own QR because a phone could not always reach
//! the Mac by name).
use std::net::{IpAddr, Ipv4Addr};

/// Where the Mac's name and address come from (the system, or a test's values).
pub trait HostInfo: Send + Sync {
    /// The Bonjour name without `.local` (e.g. "Studio-Mac").
    fn local_host_name(&self) -> Option<String>;
    /// The first private IPv4 address of an interface that is up.
    fn private_ipv4(&self) -> Option<Ipv4Addr>;
}

/// This Mac.
pub struct SystemHost;

impl HostInfo for SystemHost {
    fn local_host_name(&self) -> Option<String> {
        let out = std::process::Command::new("/usr/sbin/scutil")
            .args(["--get", "LocalHostName"])
            .output()
            .ok()?;
        let name = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        (out.status.success() && !name.is_empty()).then_some(name)
    }
    fn private_ipv4(&self) -> Option<Ipv4Addr> {
        psm_infra::net::local_addresses()
            .into_iter()
            .find_map(|a| match a {
                IpAddr::V4(v4) if v4.is_private() => Some(v4),
                _ => None,
            })
    }
}

/// The two ways to open the phone app, each with its QR code (SVG, starting with `<svg`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairingLinks {
    pub url_name: String,
    pub url_address: String,
    pub qr_name_svg: String,
    pub qr_address_svg: String,
}

/// The name the phone shows for this Mac (`hello.mac_name`).
pub fn mac_name(host: &dyn HostInfo) -> String {
    host.local_host_name().unwrap_or_else(|| "Mac".into())
}

/// `http://<name>.local:<port>/phone/?pair=<token>` and `http://<address>:<port>/phone/?pair=<token>`.
/// A missing name uses the address and a missing address the name (both missing: `localhost`).
pub fn pairing_links(host: &dyn HostInfo, port: u16, token: &str) -> PairingLinks {
    let name = host.local_host_name().map(|n| format!("{n}.local"));
    let address = host.private_ipv4().map(|a| a.to_string());
    let by_name = name
        .clone()
        .or(address.clone())
        .unwrap_or("localhost".into());
    let by_address = address.or(name).unwrap_or("localhost".into());
    let url = |host: &str| format!("http://{host}:{port}/phone/?pair={token}");
    let url_name = url(&by_name);
    let url_address = url(&by_address);
    PairingLinks {
        qr_name_svg: qr_svg(&url_name),
        qr_address_svg: qr_svg(&url_address),
        url_name,
        url_address,
    }
}

/// A QR code of `text` as an SVG element (no XML declaration, so it can be inlined).
pub fn qr_svg(text: &str) -> String {
    match qrcode::QrCode::new(text.as_bytes()) {
        Ok(code) => {
            let svg = code
                .render::<qrcode::render::svg::Color<'_>>()
                .min_dimensions(240, 240)
                .quiet_zone(true)
                .build();
            match svg.find("<svg") {
                Some(start) => svg[start..].to_owned(),
                None => svg,
            }
        }
        Err(_) => String::from(r#"<svg xmlns="http://www.w3.org/2000/svg"/>"#),
    }
}
