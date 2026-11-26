use super::{has_path, session, terminate::Target, ProcessSnapshot};
use crate::quota::Snapshot;
use reqwest::{blocking::Client, header::HeaderValue, redirect::Policy, StatusCode};
use serde::Deserialize;
use std::{
    env,
    fs::File,
    io::Read,
    mem::size_of,
    net::Ipv4Addr,
    path::{Path, PathBuf},
    ptr,
    time::{Duration, Instant, SystemTime},
};
use sysinfo::{ProcessRefreshKind, UpdateKind};
use windows_sys::Win32::{
    Foundation::{ERROR_INSUFFICIENT_BUFFER, NO_ERROR},
    NetworkManagement::IpHelper::{
        GetExtendedTcpTable, MIB_TCPROW_OWNER_PID, TCP_TABLE_OWNER_PID_LISTENER,
    },
    Networking::WinSock::AF_INET,
};
use zeroize::Zeroizing;

const CAPTURE_BUDGET: Duration = Duration::from_millis(1500);
const MAX_RESPONSE: usize = 2 * 1024 * 1024;
const MAX_HUB: u64 = 4096;
const SERVICE: &str = "exa.language_server_pb.LanguageServerService";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Hub {
    pid: u32,
    http_port: u16,
    csrf_token: Zeroizing<String>,
}

pub fn capture_quota(executable: &Path) -> Option<Snapshot> {
    let deadline = Instant::now() + CAPTURE_BUDGET;
    let processes =
        ProcessSnapshot::read(ProcessRefreshKind::new().with_exe(UpdateKind::Always)).ok()?;
    let app_pids: Vec<_> = processes
        .system
        .processes()
        .values()
        .filter(|p| {
            has_path(p, executable) && session(p.pid().as_u32()) == Some(processes.current_session)
        })
        .map(|p| p.pid().as_u32())
        .collect();
    let expected_image = executable
        .parent()?
        .join("resources")
        .join("bin")
        .join("language_server.exe");
    let mut servers = processes
        .system
        .processes()
        .values()
        .filter(|p| has_path(p, &expected_image) && processes.belongs_to_app(p, &app_pids));
    let server = servers.next()?;
    if servers.next().is_some() {
        return None;
    }
    let pid = server.pid().as_u32();
    let target = Target::observe(server).ok()?;
    let daemon = PathBuf::from(env::var_os("USERPROFILE")?)
        .join(".gemini")
        .join("antigravity")
        .join("daemon");
    let mut hubs = listener_ports(pid)?
        .into_iter()
        .filter_map(|port| read_hub(&daemon, pid, port, server.start_time()));
    let hub = hubs.next()?;
    if hubs.next().is_some() {
        return None;
    }

    let client = loopback_client()?;
    let mut csrf = HeaderValue::from_str(&hub.csrf_token).ok()?;
    csrf.set_sensitive(true);
    if !target.is_running() || !listener_ports(pid)?.contains(&hub.http_port) {
        return None;
    }
    let bytes = read_response(&client, hub.http_port, &csrf, deadline)?;
    if !target.is_running() {
        return None;
    }
    Snapshot::from_summary(&bytes)
}

fn read_hub(daemon: &Path, pid: u32, port: u16, started: u64) -> Option<Hub> {
    let file = File::open(daemon.join(format!("hub_{port}.json"))).ok()?;
    let metadata = file.metadata().ok()?;
    let modified = metadata
        .modified()
        .ok()?
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?
        .as_secs();
    if !metadata.is_file() || metadata.len() > MAX_HUB || modified < started.saturating_sub(2) {
        return None;
    }
    let mut bytes = Zeroizing::new(Vec::new());
    file.take(MAX_HUB + 1).read_to_end(&mut bytes).ok()?;
    if bytes.len() as u64 > MAX_HUB {
        return None;
    }
    let hub: Hub = serde_json::from_slice(&bytes).ok()?;
    let valid_token = hub.csrf_token.len() == 36
        && hub.csrf_token.bytes().enumerate().all(|(i, byte)| match i {
            8 | 13 | 18 | 23 => byte == b'-',
            _ => byte.is_ascii_hexdigit(),
        });
    (hub.pid == pid && hub.http_port == port && valid_token).then_some(hub)
}

fn listener_ports(pid: u32) -> Option<Vec<u16>> {
    let mut size = 0;
    // The IPv4 owner table contains a u32 count followed by aligned MIB_TCPROW_OWNER_PID rows.
    let table = |data, size| unsafe {
        GetExtendedTcpTable(
            data,
            size,
            0,
            u32::from(AF_INET),
            TCP_TABLE_OWNER_PID_LISTENER,
            0,
        )
    };
    if table(ptr::null_mut(), &mut size) != ERROR_INSUFFICIENT_BUFFER {
        return None;
    }
    // A listener may appear between table calls.
    for _ in 0..2 {
        if size > 1024 * 1024 {
            return None;
        }
        let mut data = vec![0u32; (size as usize).div_ceil(size_of::<u32>())];
        match table(data.as_mut_ptr().cast(), &mut size) {
            ERROR_INSUFFICIENT_BUFFER => continue,
            NO_ERROR => {
                let count = *data.first()? as usize;
                let available = data.len() * size_of::<u32>();
                let required = count
                    .checked_mul(size_of::<MIB_TCPROW_OWNER_PID>())?
                    .checked_add(size_of::<u32>())?;
                if required > available || required > size as usize {
                    return None;
                }
                let rows = unsafe {
                    std::slice::from_raw_parts(
                        data.as_ptr().add(1).cast::<MIB_TCPROW_OWNER_PID>(),
                        count,
                    )
                };
                return Some(
                    rows.iter()
                        .filter(|row| {
                            row.dwOwningPid == pid
                                && Ipv4Addr::from(row.dwLocalAddr.to_ne_bytes())
                                    == Ipv4Addr::LOCALHOST
                        })
                        .map(|row| u16::from_be(row.dwLocalPort as u16))
                        .collect(),
                );
            }
            _ => return None,
        }
    }
    None
}

fn loopback_client() -> Option<Client> {
    Client::builder()
        .no_proxy()
        .redirect(Policy::none())
        .retry(reqwest::retry::never())
        .build()
        .ok()
}

fn read_response(
    client: &Client,
    port: u16,
    csrf: &HeaderValue,
    deadline: Instant,
) -> Option<Zeroizing<Vec<u8>>> {
    let timeout = deadline.checked_duration_since(Instant::now())?;
    let response = client
        .post(format!(
            "http://127.0.0.1:{port}/{SERVICE}/RetrieveUserQuotaSummary"
        ))
        .timeout(timeout)
        .header("Content-Type", "application/json")
        .header("x-codeium-csrf-token", csrf.clone())
        .body("{}")
        .send()
        .ok()?;
    if response.status() != StatusCode::OK
        || response
            .content_length()
            .is_some_and(|len| len > MAX_RESPONSE as u64)
    {
        return None;
    }
    let mut bytes = Zeroizing::new(Vec::new());
    response
        .take(MAX_RESPONSE as u64 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    (bytes.len() <= MAX_RESPONSE && Instant::now() <= deadline).then_some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write, net::TcpListener, thread};

    #[test]
    fn windows_tcp_table_matches_the_bound_listener_owner_and_address() {
        let local = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let wildcard = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
        let ports = listener_ports(std::process::id()).unwrap();
        assert!(ports.contains(&local.local_addr().unwrap().port()));
        assert!(!ports.contains(&wildcard.local_addr().unwrap().port()));
    }

    #[test]
    fn quota_rpc_reads_a_summary_and_does_not_follow_redirects() {
        let target = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        target.set_nonblocking(true).unwrap();
        let redirect_port = target.local_addr().unwrap().port();
        let server = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = server.local_addr().unwrap().port();
        let stub = thread::spawn(move || {
            for response in [
                "HTTP/1.1 200 OK\r\nContent-Length: 15\r\nConnection: close\r\n\r\n{\"response\":{}}".to_owned(),
                format!(
                    "HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:{redirect_port}/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                ),
            ] {
                let (mut socket, _) = server.accept().unwrap();
                socket.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    let mut byte = [0];
                    socket.read_exact(&mut byte).unwrap();
                    request.push(byte[0]);
                    assert!(request.len() < 4096);
                }
                let mut body = [0; 2];
                socket.read_exact(&mut body).unwrap();
                assert_eq!(&body, b"{}");
                let request = String::from_utf8(request).unwrap();
                assert!(request.starts_with(&format!("POST /{SERVICE}/RetrieveUserQuotaSummary HTTP/1.1")));
                assert!(request.contains("x-codeium-csrf-token: synthetic"));
                socket.write_all(response.as_bytes()).unwrap();
            }
        });
        tauri::async_runtime::block_on(async {
            tauri::async_runtime::spawn_blocking(move || {
                let client = loopback_client().unwrap();
                let token = HeaderValue::from_static("synthetic");
                let read = || read_response(&client, port, &token, Instant::now() + CAPTURE_BUDGET);
                assert_eq!(&*read().unwrap(), br#"{"response":{}}"#);
                assert!(read().is_none());
            })
            .await
            .unwrap();
        });
        stub.join().unwrap();
        assert!(
            matches!(target.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
    }
}
