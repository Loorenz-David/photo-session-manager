use super::{ReceiverConfig, names::base_name, server::Shared};
use chrono::{TimeZone, Utc};
use psm_core::{
    domain::camera::{COMPLETED_NAMES_WINDOW_MS, Delivery, Link},
    mechanisms::hash::sha256_file,
};
use std::{
    io,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};
use tokio::{
    fs::OpenOptions,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::watch,
};

async fn reply(control: &mut BufReader<TcpStream>, code: u16, msg: &str) -> io::Result<()> {
    control
        .get_mut()
        .write_all(format!("{code} {msg}\r\n").as_bytes())
        .await
}
fn secure_eq(a: &str, b: &str) -> bool {
    let x = a.as_bytes();
    let y = b.as_bytes();
    let mut diff = x.len() ^ y.len();
    for i in 0..x.len().max(y.len()) {
        diff |= usize::from(x.get(i).copied().unwrap_or(0) ^ y.get(i).copied().unwrap_or(0));
    }
    diff == 0
}
struct UploadGuard {
    state: Arc<Shared>,
    name: String,
}
impl Drop for UploadGuard {
    fn drop(&mut self) {
        self.state.active.lock().unwrap().remove(&self.name);
        self.state.uploads.fetch_sub(1, Ordering::SeqCst);
        self.state
            .config
            .sink
            .transfer_in_progress(Link::Wireless, false);
    }
}
fn completed(config: &ReceiverConfig) -> Vec<(String, u64, i64)> {
    config
        .names
        .completed_names_since(config.clock.now_ms() - COMPLETED_NAMES_WINDOW_MS)
}
async fn passive(config: &ReceiverConfig) -> io::Result<TcpListener> {
    for port in config.passive.clone() {
        if let Ok(listener) = TcpListener::bind(SocketAddr::new(config.bind, port)).await {
            return Ok(listener);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AddrInUse,
        "passive ports occupied",
    ))
}
async fn accept_data(listener: TcpListener, peer: IpAddr) -> io::Result<TcpStream> {
    let (stream, addr) =
        tokio::time::timeout(Duration::from_secs(600), listener.accept()).await??;
    if addr.ip() != peer {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "data peer mismatch",
        ));
    }
    Ok(stream)
}
async fn upload(
    listener: TcpListener,
    peer: IpAddr,
    part: PathBuf,
    name: String,
    offset: Option<u64>,
    append: bool,
    state: Arc<Shared>,
) -> io::Result<()> {
    let mut data = accept_data(listener, peer).await?;
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&part)
        .await?;
    if let Some(offset) = offset {
        file.set_len(offset).await?;
        file.seek(std::io::SeekFrom::Start(offset)).await?;
    } else if append {
        file.seek(std::io::SeekFrom::End(0)).await?;
    } else {
        file.set_len(0).await?;
        file.seek(std::io::SeekFrom::Start(0)).await?;
    }
    let mut buffer = [0_u8; 65536];
    loop {
        let count = tokio::time::timeout(Duration::from_secs(600), async {
            loop {
                tokio::select! {
                    read = data.read(&mut buffer) => return read,
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {
                        if let Some(error) = data.take_error()? { return Err(error); }
                    }
                }
            }
        })
        .await??;
        if count == 0 {
            break;
        }
        file.write_all(&buffer[..count]).await?;
    }
    if data.take_error()?.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::ConnectionReset,
            "pending data socket error",
        ));
    }
    file.flush().await?;
    let completed_at = state.config.clock.now_ms();
    drop(file);
    let size = tokio::fs::metadata(&part).await?.len();
    let hash_path = part.clone();
    let sha256 = tokio::task::spawn_blocking(move || sha256_file(&hash_path))
        .await
        .map_err(io::Error::other)?
        .map_err(io::Error::other)?;
    state
        .config
        .sink
        .delivery(Delivery {
            camera_filename: name,
            written_path: part,
            sha256,
            size,
            via: Link::Wireless,
            completed_at,
        })
        .map_err(io::Error::other)?;
    Ok(())
}

pub(crate) async fn serve(
    stream: TcpStream,
    peer: SocketAddr,
    state: Arc<Shared>,
    mut stop: watch::Receiver<bool>,
) -> io::Result<()> {
    let mut control = BufReader::new(stream);
    reply(&mut control, 220, "Session Manager FTP ready").await?;
    let mut user = String::new();
    let mut authenticated = false;
    let mut dir = "/".to_owned();
    let mut pasv: Option<TcpListener> = None;
    let mut rest: Option<u64> = None;
    loop {
        let mut line = String::new();
        tokio::select! {
            _ = stop.changed() => break,
            result = control.read_line(&mut line) => { if result? == 0 { break; } }
        }
        let line = line.trim_end_matches(['\r', '\n']);
        let (raw, arg) = line.split_once(' ').unwrap_or((line, ""));
        let cmd = raw.to_ascii_uppercase();
        if !authenticated
            && !matches!(
                cmd.as_str(),
                "USER" | "PASS" | "QUIT" | "FEAT" | "SYST" | "NOOP"
            )
        {
            reply(&mut control, 530, "Log in first").await?;
            continue;
        }
        match cmd.as_str() {
            "USER" => {
                user = arg.to_owned();
                authenticated = false;
                reply(&mut control, 331, "Password required").await?;
            }
            "PASS" => {
                authenticated =
                    user == state.config.username && secure_eq(arg, &state.config.password);
                if authenticated {
                    reply(&mut control, 230, "Logged in").await?;
                } else {
                    tracing::warn!(
                        user = if user == state.config.username {
                            "sessionmanager"
                        } else {
                            "<other>"
                        },
                        "FTP login failed"
                    );
                    reply(&mut control, 530, "Login incorrect").await?;
                }
            }
            "QUIT" => {
                reply(&mut control, 221, "Goodbye").await?;
                break;
            }
            "NOOP" => reply(&mut control, 200, "OK").await?,
            "SYST" => reply(&mut control, 215, "UNIX Type: L8").await?,
            "FEAT" => {
                control
                    .get_mut()
                    .write_all(b"211-Features\r\n EPSV\r\n REST STREAM\r\n UTF8\r\n211 End\r\n")
                    .await?
            }
            "OPTS" if arg.eq_ignore_ascii_case("UTF8 ON") => {
                reply(&mut control, 200, "UTF8 on").await?
            }
            "TYPE" => reply(&mut control, 200, "Type set").await?,
            "PWD" => reply(&mut control, 257, &format!("\"{dir}\"")).await?,
            "CWD" => {
                dir = if arg.starts_with('/') {
                    arg.to_owned()
                } else {
                    format!("{}/{}", dir.trim_end_matches('/'), arg)
                };
                reply(&mut control, 250, "Directory changed").await?;
            }
            "CDUP" => {
                dir = "/".into();
                reply(&mut control, 250, "Directory changed").await?;
            }
            "MKD" => reply(&mut control, 257, &format!("\"{arg}\" created")).await?,
            "PASV" | "EPSV" => match passive(&state.config).await {
                Ok(listener) => {
                    let port = listener.local_addr()?.port();
                    pasv = Some(listener);
                    if cmd == "EPSV" {
                        reply(
                            &mut control,
                            229,
                            &format!("Entering Extended Passive Mode (|||{port}|)"),
                        )
                        .await?;
                    } else {
                        let octets = match state.config.bind {
                            IpAddr::V4(v) => v.octets(),
                            _ => [127, 0, 0, 1],
                        };
                        reply(
                            &mut control,
                            227,
                            &format!(
                                "Entering Passive Mode ({},{},{},{},{},{})",
                                octets[0],
                                octets[1],
                                octets[2],
                                octets[3],
                                port / 256,
                                port % 256
                            ),
                        )
                        .await?;
                    }
                }
                Err(_) => reply(&mut control, 425, "No passive port available").await?,
            },
            "REST" => {
                if let Ok(offset) = arg.parse::<u64>() {
                    rest = Some(offset);
                    reply(&mut control, 350, "Restart position accepted").await?;
                } else {
                    reply(&mut control, 501, "Invalid offset").await?;
                }
            }
            "STOR" | "APPE" => {
                let resume = rest.take();
                let Ok(name) = base_name(arg) else {
                    reply(&mut control, 553, "Invalid name").await?;
                    continue;
                };
                let key = name.to_uppercase();
                let Some(root) = (state.config.capture_root)().filter(|p| p.is_dir()) else {
                    reply(&mut control, 451, "Capture folder unavailable").await?;
                    continue;
                };
                let reserved = { state.active.lock().unwrap().insert(key.clone()) };
                if !reserved {
                    reply(&mut control, 450, "File busy").await?;
                    continue;
                }
                state.uploads.fetch_add(1, Ordering::SeqCst);
                state.config.sink.transfer_in_progress(Link::Wireless, true);
                let guard = UploadGuard {
                    state: state.clone(),
                    name: key,
                };
                let incoming = root.join(".incoming");
                if tokio::fs::create_dir_all(&incoming).await.is_err() {
                    drop(guard);
                    reply(&mut control, 451, "Cannot stage file").await?;
                    continue;
                }
                let part = incoming.join(format!(".{name}.part"));
                let append = cmd == "APPE";
                let offset = if append && resume.is_none() {
                    Some(
                        tokio::fs::metadata(&part)
                            .await
                            .map(|m| m.len())
                            .unwrap_or(0),
                    )
                } else {
                    resume
                };
                if let Some(n) = offset
                    && !tokio::fs::metadata(&part).await.is_ok_and(|m| m.len() >= n)
                {
                    drop(guard);
                    reply(&mut control, 554, "Restart position unavailable").await?;
                    continue;
                }
                let Some(listener) = pasv.take() else {
                    drop(guard);
                    reply(&mut control, 425, "Use passive mode").await?;
                    continue;
                };
                reply(&mut control, 150, "Opening data connection").await?;
                let mut pending_line = String::new();
                let result = tokio::select! {
                    r = upload(listener, peer.ip(), part, name, offset, append, state.clone()) => r,
                    _ = stop.changed() => Err(io::Error::new(io::ErrorKind::Interrupted, "stopped")),
                    r = control.read_line(&mut pending_line) => { let _ = r; Err(io::Error::new(io::ErrorKind::Interrupted, "control closed or aborted")) }
                };
                drop(guard);
                if *stop.borrow() {
                    break;
                }
                if result.is_ok() {
                    reply(&mut control, 226, "Transfer complete").await?;
                } else {
                    reply(&mut control, 426, "Transfer aborted").await?;
                }
            }
            "LIST" | "NLST" => {
                let Some(listener) = pasv.take() else {
                    reply(&mut control, 425, "Use passive mode").await?;
                    continue;
                };
                reply(&mut control, 150, "Opening data connection").await?;
                let result = async {
                    let mut data = accept_data(listener, peer.ip()).await?;
                    for (name, size, at) in completed(&state.config) {
                        let row = if cmd == "NLST" {
                            name
                        } else {
                            let date = Utc
                                .timestamp_millis_opt(at)
                                .single()
                                .unwrap_or_else(Utc::now)
                                .format("%b %d %H:%M");
                            format!("-rw-r--r-- 1 camera camera {size} {date} {name}")
                        };
                        data.write_all(format!("{row}\r\n").as_bytes()).await?;
                    }
                    data.shutdown().await
                }
                .await;
                if result.is_ok() {
                    reply(&mut control, 226, "Transfer complete").await?;
                } else {
                    reply(&mut control, 426, "Transfer aborted").await?;
                }
            }
            "SIZE" | "MDTM" => {
                let name = base_name(arg).ok();
                let entry = completed(&state.config)
                    .into_iter()
                    .find(|(n, _, _)| Some(n.as_str()) == name.as_deref());
                if let Some((_, size, at)) = entry {
                    if cmd == "SIZE" {
                        reply(&mut control, 213, &size.to_string()).await?;
                    } else {
                        let stamp = Utc
                            .timestamp_millis_opt(at)
                            .single()
                            .unwrap_or_else(Utc::now)
                            .format("%Y%m%d%H%M%S");
                        reply(&mut control, 213, &stamp.to_string()).await?;
                    }
                } else {
                    reply(&mut control, 550, "Not found").await?;
                }
            }
            "DELE" | "RMD" | "RNFR" => reply(&mut control, 550, "Not permitted").await?,
            "RNTO" => reply(&mut control, 553, "Not permitted").await?,
            _ => reply(&mut control, 502, "Command not implemented").await?,
        }
    }
    Ok(())
}
