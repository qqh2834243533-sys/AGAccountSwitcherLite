use serde::{Deserialize, Serialize};
use std::{
    fs,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use sysinfo::{Pid, Process, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use windows_sys::Win32::{
    Storage::FileSystem::FILE_SHARE_READ, System::RemoteDesktop::ProcessIdToSessionId,
};

mod graceful;
mod installation;
mod terminate;

use graceful::GracefulExit;
pub use installation::installed;

const LAUNCH_CONTEXT_FILE: &str = "launch-context.json";

#[derive(Default)]
pub struct LaunchPlan {
    app_args: Vec<String>,
    companion: Option<ObservedCompanion>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedLaunchContext {
    #[serde(default)]
    app_args: Vec<String>,
}

struct ObservedCompanion {
    executable: PathBuf,
    args: Vec<String>,
    spawns_app: bool,
    pid: u32,
    start_time: u64,
    // Keep the observed image open without write/delete sharing until relaunch.
    _image: fs::File,
}

fn session(pid: u32) -> Option<u32> {
    let mut id = 0;
    (unsafe { ProcessIdToSessionId(pid, &mut id) } != 0).then_some(id)
}

fn is_core_antigravity_writer_name(name_lower: &str) -> bool {
    matches!(
        name_lower,
        "antigravity.exe"
            | "antigravity ide.exe"
            | "antigravity-ide.exe"
            | "antigravity_ide.exe"
            | "agy.exe"
    )
}

fn is_antigravity_language_server_path(exe_path: Option<&Path>) -> bool {
    let Some(path) = exe_path else {
        // Fail closed if the OS cannot report the executable image path.
        return true;
    };
    let normalized = path
        .to_string_lossy()
        .to_ascii_lowercase()
        .replace('\\', "/");
    normalized.contains("/antigravity/")
        || normalized.contains("/.gemini/antigravity")
        || normalized.contains("/antigravity ide/")
        || normalized.contains("/antigravity-ide/")
        || normalized.contains("/antigravity_ide/")
}

struct ProcessSnapshot {
    system: System,
    current_session: u32,
}

impl ProcessSnapshot {
    fn read(kind: ProcessRefreshKind) -> Result<Self, &'static str> {
        let pid = std::process::id();
        let current_session = session(pid).ok_or("client_state_unknown")?;
        let mut system = System::new();
        system.refresh_processes_specifics(ProcessesToUpdate::All, kind);
        if system.process(Pid::from_u32(pid)).is_none() {
            return Err("client_state_unknown");
        }
        Ok(Self {
            system,
            current_session,
        })
    }

    fn has_antigravity_ancestor<'a>(&'a self, mut process: &'a Process) -> bool {
        for _ in 0..32 {
            let Some(parent) = process.parent().and_then(|pid| self.system.process(pid)) else {
                return false;
            };
            if parent.start_time() > process.start_time() {
                return false;
            }
            if session(parent.pid().as_u32()).is_some_and(|id| id != self.current_session) {
                return false;
            }
            let parent_name = parent.name().to_string_lossy().to_ascii_lowercase();
            if is_core_antigravity_writer_name(&parent_name)
                && !(parent_name == "antigravity.exe" && is_sidecar_node_process(parent))
            {
                return true;
            }
            process = parent;
        }
        false
    }

    // Match Windows shared-state writers. Keep unknown sessions in the result so a
    // failed observation cannot look like a clean exit.
    // Exclude Node.js plugin sidecar workers (`Antigravity.exe --import=.../sidecar_sdk/...`)
    // and unrelated third-party IDE `language_server.exe` processes.
    fn writers<'a>(&'a self, executable: Option<&'a Path>) -> impl Iterator<Item = &'a Process> {
        self.system.processes().values().filter(move |process| {
            if !session(process.pid().as_u32()).is_none_or(|id| id == self.current_session) {
                return false;
            }
            let name = process.name().to_string_lossy().to_ascii_lowercase();
            if is_core_antigravity_writer_name(&name) {
                return !(name == "antigravity.exe" && is_sidecar_node_process(process));
            }
            if name == "language_server.exe" {
                return is_antigravity_language_server_path(process.exe())
                    || process
                        .exe()
                        .zip(executable.and_then(Path::parent))
                        .is_some_and(|(image, root)| {
                            image.ancestors().any(|dir| same_image_path(dir, root))
                        })
                    || self.has_antigravity_ancestor(process);
            }
            false
        })
    }

    fn belongs_to_app<'a>(&'a self, mut process: &'a Process, app_pids: &[u32]) -> bool {
        if session(process.pid().as_u32()) != Some(self.current_session) {
            return false;
        }
        for _ in 0..32 {
            if app_pids.contains(&process.pid().as_u32()) {
                return true;
            }
            let Some(parent) = process.parent().and_then(|pid| self.system.process(pid)) else {
                return false;
            };
            if parent.start_time() > process.start_time() {
                return false;
            }
            process = parent;
        }
        false
    }

    fn main_app_process<'a>(&'a self, executable: &Path, app_pids: &[u32]) -> Option<&'a Process> {
        let mut candidates: Vec<&Process> = app_pids
            .iter()
            .filter_map(|pid| self.system.process(Pid::from_u32(*pid)))
            .filter(|process| has_path(process, executable))
            .filter(|process| {
                let args = process_args(process);
                !is_electron_child_args(&args)
            })
            .collect();

        // Prefer a root Antigravity.exe whose parent is not itself in app_pids.
        if let Some(root) = candidates.iter().copied().find(|process| {
            process
                .parent()
                .is_none_or(|parent| !app_pids.contains(&parent.as_u32()))
        }) {
            return Some(root);
        }

        candidates.sort_by_key(|process| (process.start_time(), process.pid().as_u32()));
        candidates.into_iter().next()
    }

    fn detect_companion(&self, main_app: &Process, executable: &Path) -> Option<ObservedCompanion> {
        // Tier 1: walk up to 2 ancestor levels from the main Antigravity process
        // (e.g. a plugin shortcut running `watcher.exe --launch --exit-on-host-close`).
        let mut child = main_app;
        for _ in 0..2 {
            let Some(parent) = child
                .parent()
                .and_then(|pid| self.system.process(pid))
                .filter(|parent| parent.start_time() <= child.start_time())
                .filter(|parent| session(parent.pid().as_u32()) == Some(self.current_session))
            else {
                break;
            };
            if let Some(companion) = build_companion_from_process(parent, executable, true) {
                return Some(companion);
            }
            child = parent;
        }

        // Tier 2: attach-mode companions in the same Windows session configured with
        // `--exit-on-host-close` AND explicit Antigravity / AGY plugin affinity
        // (such as `agy-claude-style-watcher`), selected deterministically.
        self.system
            .processes()
            .values()
            .filter(|process| session(process.pid().as_u32()) == Some(self.current_session))
            .filter_map(|process| build_companion_from_process(process, executable, false))
            .max_by_key(|companion| (companion.start_time, companion.pid))
    }

    fn capture_launch_plan(&self, executable: &Path, app_pids: &[u32]) -> Option<LaunchPlan> {
        let main_app = self.main_app_process(executable, app_pids)?;
        let app_args = filter_relaunch_app_args(&process_args(main_app));
        let companion = self.detect_companion(main_app, executable);
        Some(LaunchPlan {
            app_args,
            companion,
        })
    }
}

fn process_args(process: &Process) -> Vec<String> {
    process
        .cmd()
        .iter()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

fn is_sidecar_node_args(args: &[String]) -> bool {
    args.iter().any(|arg| {
        let lower = arg.to_ascii_lowercase().replace('\\', "/");
        lower.starts_with("--import=") && lower.contains("sidecar_sdk")
    })
}

fn is_sidecar_node_process(process: &Process) -> bool {
    let args = process_args(process);
    !args.is_empty() && is_sidecar_node_args(&args)
}

fn is_electron_child_args(args: &[String]) -> bool {
    is_sidecar_node_args(args)
        || args.iter().any(|arg| {
            let lower = arg.to_ascii_lowercase();
            lower.starts_with("--type=")
                || lower.starts_with("--import=")
                || lower.starts_with("--crash-reporter")
                || lower == "--node-ipc"
        })
}

fn loopback_address(value: &str) -> bool {
    matches!(
        value.parse::<IpAddr>(),
        Ok(IpAddr::V4(Ipv4Addr::LOCALHOST) | IpAddr::V6(Ipv6Addr::LOCALHOST))
    )
}

fn valid_local_proxy(value: &str) -> bool {
    let authority = match value.split_once("://") {
        Some(("http" | "https" | "socks4" | "socks5", authority)) => authority,
        Some(_) => return false,
        None => value,
    };
    let Some((host, port)) = authority.rsplit_once(':') else {
        return false;
    };
    matches!(host, "localhost" | "127.0.0.1" | "[::1]")
        && port.parse::<u16>().is_ok_and(|port| port != 0)
}

fn valid_relaunch_value(flag: &str, value: &str) -> bool {
    match flag {
        "--remote-debugging-port" => value.parse::<u16>().is_ok(),
        "--remote-debugging-address" => loopback_address(value),
        "--proxy-server" => valid_local_proxy(value),
        "--lang" | "--locale" => {
            (2..=35).contains(&value.len())
                && value.as_bytes()[..2].iter().all(u8::is_ascii_alphabetic)
                && value.split('-').all(|part| {
                    !part.is_empty() && part.bytes().all(|ch| ch.is_ascii_alphanumeric())
                })
        }
        "--force-device-scale-factor" => value
            .parse::<f32>()
            .is_ok_and(|scale| (0.5..=5.0).contains(&scale)),
        _ => false,
    }
}

/// Replay only documented launch options with validated values.
fn filter_relaunch_app_args(args: &[String]) -> Vec<String> {
    let mut kept = Vec::new();
    let mut args = args.iter().peekable();
    while let Some(arg) = args.next() {
        if arg == "--" {
            break;
        }
        if let Some((flag, value)) = arg.split_once('=') {
            if valid_relaunch_value(flag, value) {
                kept.push(arg.clone());
            }
        } else if matches!(
            arg.as_str(),
            "--disable-gpu" | "--disable-gpu-compositing" | "--disable-software-rasterizer"
        ) {
            kept.push(arg.clone());
        } else if args
            .peek()
            .is_some_and(|value| valid_relaunch_value(arg, value))
        {
            kept.push(arg.clone());
            kept.extend(args.next().cloned());
        }
    }
    kept
}

fn is_shell_or_self_name(name_lower: &str) -> bool {
    matches!(
        name_lower,
        "explorer.exe"
            | "cmd.exe"
            | "powershell.exe"
            | "pwsh.exe"
            | "wt.exe"
            | "windowsterminal.exe"
            | "conhost.exe"
            | "openconsole.exe"
            | "svchost.exe"
            | "services.exe"
            | "sihost.exe"
            | "runtimebroker.exe"
            | "ag-account-switcher-lite.exe"
            | "antigravity.exe"
            | "antigravity ide.exe"
            | "antigravity-ide.exe"
            | "antigravity_ide.exe"
            | "agy.exe"
    )
}

fn has_antigravity_token(text: &str) -> bool {
    let normalized = text.to_ascii_lowercase().replace('\\', "/");
    normalized.contains("antigravity")
        || normalized.contains("agy-")
        || normalized.contains("/agy_")
}

fn has_antigravity_affinity(executable: &Path, args: &[String]) -> bool {
    has_antigravity_token(&executable.to_string_lossy())
        || args.iter().any(|arg| has_antigravity_token(arg))
}

fn evaluate_companion_candidate(
    executable: &Path,
    raw_args: &[String],
    verified_live_ancestor: bool,
) -> Option<(Vec<String>, bool)> {
    let file_name = executable
        .file_name()?
        .to_string_lossy()
        .to_ascii_lowercase();
    if is_shell_or_self_name(&file_name) {
        return None;
    }
    let has_exit_on_close = raw_args
        .iter()
        .any(|arg| arg.eq_ignore_ascii_case("--exit-on-host-close"));
    let spawns_app = raw_args
        .iter()
        .any(|arg| arg.eq_ignore_ascii_case("--launch"));

    // Manage processes that die with the host (`--exit-on-host-close`) or are a direct
    // ancestor plugin launcher (`--launch`) wrapping Antigravity.
    // For attach-mode processes outside the parent chain, require explicit Antigravity /
    // AGY plugin affinity so unrelated session processes are never captured.
    if verified_live_ancestor {
        if !has_exit_on_close && !spawns_app {
            return None;
        }
    } else if spawns_app || !has_exit_on_close || !has_antigravity_affinity(executable, raw_args) {
        return None;
    }

    let args = ["--launch", "--exit-on-host-close"]
        .into_iter()
        .filter(|flag| raw_args.iter().any(|arg| arg.eq_ignore_ascii_case(flag)))
        .map(str::to_owned)
        .collect();
    Some((args, spawns_app))
}

fn build_companion_from_process(
    process: &Process,
    app_executable: &Path,
    verified_live_ancestor: bool,
) -> Option<ObservedCompanion> {
    if process.pid().as_u32() == std::process::id() || has_path(process, app_executable) {
        return None;
    }
    let executable = process.exe()?;
    let raw_args = process_args(process);
    let (args, spawns_app) =
        evaluate_companion_candidate(executable, &raw_args, verified_live_ancestor)?;
    let executable = fs::canonicalize(executable).ok()?;
    if is_shell_or_self_name(
        &executable
            .file_name()?
            .to_string_lossy()
            .to_ascii_lowercase(),
    ) {
        return None;
    }
    let image = fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(&executable)
        .ok()?;
    Some(ObservedCompanion {
        executable,
        args,
        spawns_app,
        pid: process.pid().as_u32(),
        start_time: process.start_time(),
        _image: image,
    })
}

fn load_launch_plan(data_root: &Path) -> LaunchPlan {
    let path = data_root.join(LAUNCH_CONTEXT_FILE);
    let Ok(raw) = fs::read(&path) else {
        return LaunchPlan::default();
    };
    let app_args = serde_json::from_slice::<SavedLaunchContext>(&raw)
        .map(|saved| filter_relaunch_app_args(&saved.app_args))
        .unwrap_or_default();
    // Rewrite old files once to remove legacy companion and rejected arguments.
    save_launch_args(data_root, &app_args, Some(&raw));
    LaunchPlan {
        app_args,
        companion: None,
    }
}

fn save_launch_args(data_root: &Path, args: &[String], previous: Option<&[u8]>) {
    let saved = SavedLaunchContext {
        app_args: filter_relaunch_app_args(args),
    };
    let Ok(serialized) = serde_json::to_vec_pretty(&saved) else {
        return;
    };
    let final_path = data_root.join(LAUNCH_CONTEXT_FILE);
    let current = previous
        .map(std::borrow::Cow::Borrowed)
        .or_else(|| fs::read(&final_path).ok().map(std::borrow::Cow::Owned));
    if current.as_deref() != Some(serialized.as_slice()) {
        let _ = crate::config_file::replace(&final_path, &serialized);
    }
}

pub fn inspect_running(executable: Option<&Path>, data_root: &Path) -> bool {
    let Ok(snapshot) = ProcessSnapshot::read(
        ProcessRefreshKind::new()
            .with_exe(UpdateKind::Always)
            .with_cmd(UpdateKind::Always),
    ) else {
        return true;
    };
    let mut writers = snapshot.writers(executable).peekable();
    let running = writers.peek().is_some();
    if let Some(executable) = executable {
        let app_pids: Vec<u32> = writers
            .filter(|process| has_path(process, executable))
            .filter(|process| session(process.pid().as_u32()) == Some(snapshot.current_session))
            .map(|process| process.pid().as_u32())
            .collect();
        if let Some(app) = snapshot.main_app_process(executable, &app_pids) {
            save_launch_args(data_root, &process_args(app), None);
        }
    }
    running
}

pub fn require_closed() -> Result<(), &'static str> {
    require_closed_for(installed().as_deref())
}

pub fn require_closed_for(executable: Option<&Path>) -> Result<(), &'static str> {
    if ProcessSnapshot::read(
        ProcessRefreshKind::new()
            .with_exe(UpdateKind::Always)
            .with_cmd(UpdateKind::Always),
    )?
    .writers(executable)
    .next()
    .is_some()
    {
        return Err("client_running");
    }
    Ok(())
}

fn same_image_path(first: &Path, second: &Path) -> bool {
    let first = first.to_string_lossy();
    let second = second.to_string_lossy();
    first
        .trim_start_matches(r"\\?\")
        .eq_ignore_ascii_case(second.trim_start_matches(r"\\?\"))
}

fn has_path(process: &Process, executable: &Path) -> bool {
    process
        .exe()
        .is_some_and(|path| same_image_path(path, executable))
}

pub fn prepare_switch(executable: &Path, data_root: &Path) -> Result<LaunchPlan, &'static str> {
    let snapshot = ProcessSnapshot::read(
        ProcessRefreshKind::new()
            .with_exe(UpdateKind::Always)
            .with_cmd(UpdateKind::Always),
    )?;
    let writers: Vec<_> = snapshot.writers(Some(executable)).collect();
    if writers.is_empty() {
        return Ok(load_launch_plan(data_root));
    }
    let app_pids: Vec<u32> = writers
        .iter()
        .filter(|process| has_path(process, executable))
        .filter(|process| session(process.pid().as_u32()) == Some(snapshot.current_session))
        .map(|process| process.pid().as_u32())
        .collect();
    if app_pids.is_empty()
        || writers
            .iter()
            .any(|process| !snapshot.belongs_to_app(process, &app_pids))
    {
        return Err("client_running");
    }

    let targets: Vec<_> = writers
        .iter()
        .map(|process| terminate::Target::open(process))
        .collect::<Result<_, _>>()?;

    let plan = snapshot
        .capture_launch_plan(executable, &app_pids)
        .unwrap_or_else(|| load_launch_plan(data_root));
    save_launch_args(data_root, &plan.app_args, None);

    // Phase 1: silently close visible windows via WM_CLOSE so renderer state flushes
    // cleanly without touching the tray, mouse, or keyboard focus.
    // If any visible window refuses to close (e.g. unsaved dialog), this returns
    // `Err("close_timeout")` and never enters Phase 2.
    let closed = || require_closed_for(Some(executable));
    let exit_state =
        graceful::close_windows_gracefully(&app_pids, closed, Duration::from_millis(2500))?;

    // Phase 2: only if all visible windows closed/hid to tray (`HiddenToTray`) while
    // tray-resident processes remain, silently terminate the verified processes.
    let deadline = Instant::now() + Duration::from_secs(12);
    if exit_state == GracefulExit::HiddenToTray {
        terminate::remaining(&targets, deadline)?;
    }
    loop {
        match closed() {
            Ok(()) => {
                graceful::clear_stale_devtools_port();
                // Give Windows / Electron a brief moment to release single-instance
                // locks and flush handles before relaunching the new process.
                thread::sleep(Duration::from_millis(250));
                return Ok(plan);
            }
            Err("client_running") if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(150));
            }
            Err("client_running") => return Err("close_timeout"),
            Err(reason) => return Err(reason),
        }
    }
}

fn matches_executable(system: &mut System, executable: &Path, current_session: u32) -> bool {
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        ProcessRefreshKind::new()
            .with_exe(UpdateKind::Always)
            .with_cmd(UpdateKind::Always),
    );
    system.processes().values().any(|process| {
        has_path(process, executable)
            && !is_sidecar_node_process(process)
            && session(process.pid().as_u32()) == Some(current_session)
    })
}

fn companion_is_still_running(companion: &ObservedCompanion, current_session: u32) -> bool {
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        ProcessRefreshKind::new().with_exe(UpdateKind::Always),
    );
    // Also avoid a duplicate if the observed companion restarted itself.
    system.processes().values().any(|process| {
        has_path(process, &companion.executable)
            && session(process.pid().as_u32()) == Some(current_session)
    })
}

fn spawn_detached(
    executable: &Path,
    args: &[String],
    directory: &Path,
) -> Result<std::process::Child, &'static str> {
    Command::new(executable)
        .args(args)
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "launch_failed")
}

fn wait_for_app_running(
    executable: &Path,
    current_session: u32,
    mut child: Option<&mut std::process::Child>,
    timeout: Duration,
) -> Result<(), &'static str> {
    let deadline = Instant::now() + timeout;
    let mut system = System::new();
    let mut visible_process_since = None;
    while Instant::now() < deadline {
        if matches_executable(&mut system, executable, current_session) {
            let started = *visible_process_since.get_or_insert_with(Instant::now);
            if started.elapsed() >= Duration::from_millis(500) {
                return Ok(());
            }
        } else {
            visible_process_since = None;
            if let Some(ref mut process) = child {
                if process
                    .try_wait()
                    .ok()
                    .flatten()
                    .is_some_and(|status| !status.success())
                {
                    return Err("launch_failed");
                }
            }
        }
        thread::sleep(Duration::from_millis(150));
    }
    Err("launch_failed")
}

pub fn open(executable: &Path, plan: &LaunchPlan) -> Result<(), &'static str> {
    let current_session = session(std::process::id()).ok_or("client_state_unknown")?;
    let directory = executable.parent().ok_or("app_missing")?;
    let safe_app_args = filter_relaunch_app_args(&plan.app_args);

    // If Antigravity was previously launched via a companion launcher (e.g. a plugin
    // shortcut with `--launch --exit-on-host-close`) and that launcher exited alongside
    // Antigravity, re-invoke the companion so both Antigravity and the plugin recover.
    if let Some(companion) = &plan.companion {
        if companion.spawns_app && !companion_is_still_running(companion, current_session) {
            let companion_dir = companion.executable.parent().unwrap_or(directory);
            if let Ok(mut child) =
                spawn_detached(&companion.executable, &companion.args, companion_dir)
            {
                if wait_for_app_running(
                    executable,
                    current_session,
                    Some(&mut child),
                    Duration::from_secs(7),
                )
                .is_ok()
                {
                    return Ok(());
                }
            }
        }
    }

    // Standard launch path: preserve sanitized main-process flags (such as
    // `--remote-debugging-port=0`) so resident CDP watchers re-attach automatically.
    let mut child = spawn_detached(executable, &safe_app_args, directory)?;
    wait_for_app_running(
        executable,
        current_session,
        Some(&mut child),
        Duration::from_secs(10),
    )?;

    // If an attach-only companion (`--exit-on-host-close` without `--launch`, with
    // verified Antigravity/AGY affinity) exited with the old Antigravity process,
    // best-effort relaunch it now that the host is up.
    if let Some(companion) = &plan.companion {
        if !companion.spawns_app && !companion_is_still_running(companion, current_session) {
            let companion_dir = companion.executable.parent().unwrap_or(directory);
            let _ = spawn_detached(&companion.executable, &companion.args, companion_dir);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn filters_electron_child_processes_and_sanitizes_relaunch_app_args() {
        assert!(is_electron_child_args(&[
            "--type=renderer".into(),
            "--remote-debugging-port=0".into()
        ]));
        assert!(is_electron_child_args(&[
            "--import=file:///C:/Users/example/.gemini/antigravity/lib/node_modules/sidecar_sdk/register_loader.js".into(),
            "main.mjs".into()
        ]));
        assert!(is_sidecar_node_args(&[
            "--import=file:///C:/Users/example/.gemini/antigravity/lib/node_modules/sidecar_sdk/register_loader.js".into(),
            "main.mjs".into()
        ]));
        assert!(is_sidecar_node_args(&[
            r"--import=C:\Users\example\.gemini\antigravity\lib\node_modules\sidecar_sdk\register_loader.js".into(),
            "main.mjs".into()
        ]));
        assert!(!is_sidecar_node_args(&[
            "--remote-debugging-port=0".into(),
            "--lang=zh-CN".into()
        ]));
        assert!(!is_sidecar_node_args(&[]));
        assert!(!is_electron_child_args(&[
            "--remote-debugging-port=0".into(),
            "--lang=zh-CN".into()
        ]));

        let filtered = filter_relaunch_app_args(&[
            "--remote-debugging-port=0".into(),
            "--squirrel-firstrun".into(),
            "--new-window".into(),
            r"C:\Temp\TransientWorkspace".into(),
            "antigravity://oauth/callback?code=secret".into(),
            "--proxy-server=http://127.0.0.1:7890".into(),
            "--force-device-scale-factor=1.25".into(),
            "--disable-gpu".into(),
            "--import=file:///C:/evil.mjs".into(),
            "--inspect=9229".into(),
            "--install-extension".into(),
            "some.ext".into(),
            "--unknown-flag=test".into(),
            "--remote-debugging-address=0.0.0.0".into(),
            "--lang=absolute/path".into(),
            "--proxy-server=http://proxy.example.test:8080".into(),
        ]);
        assert_eq!(
            filtered,
            vec![
                "--remote-debugging-port=0".to_string(),
                "--proxy-server=http://127.0.0.1:7890".to_string(),
                "--force-device-scale-factor=1.25".to_string(),
                "--disable-gpu".to_string(),
            ]
        );

        let two_token = filter_relaunch_app_args(&[
            "--remote-debugging-port".into(),
            "9222".into(),
            "--lang".into(),
            "zh-CN".into(),
            r"D:\LooseFile.md".into(),
        ]);
        assert_eq!(
            two_token,
            vec![
                "--remote-debugging-port".to_string(),
                "9222".to_string(),
                "--lang".to_string(),
                "zh-CN".to_string(),
            ]
        );
    }

    #[test]
    fn enforces_plugin_affinity_for_attach_companions_and_rejects_unrelated_daemons() {
        // 1. Attach-mode `agy-claude-style-watcher` matches via path/name affinity.
        let style_watcher = evaluate_companion_candidate(
            Path::new(
                r"C:\Users\Alice\AppData\Local\agy-claude-style\agy-claude-style-watcher.exe",
            ),
            &["--exit-on-host-close".into(), "--relaunch".into()],
            false,
        );
        assert_eq!(
            style_watcher,
            Some((vec!["--exit-on-host-close".to_string()], false))
        );

        // 2. Attach-mode generic binary with explicit Antigravity arg affinity matches.
        let arg_affiliated = evaluate_companion_candidate(
            Path::new(r"C:\Tools\cdp-helper.exe"),
            &[
                "--exit-on-host-close".into(),
                "--target".into(),
                "antigravity".into(),
            ],
            false,
        );
        assert!(arg_affiliated.is_some());

        // 3. Unrelated session daemon with `--exit-on-host-close` is rejected.
        let unrelated = evaluate_companion_candidate(
            Path::new(r"C:\Program Files\OtherVendor\background-helper.exe"),
            &["--exit-on-host-close".into()],
            false,
        );
        assert_eq!(unrelated, None);

        // 4. Shell binaries are always rejected even as direct ancestors.
        let shell = evaluate_companion_candidate(
            Path::new(r"C:\Windows\System32\cmd.exe"),
            &["--launch".into(), "--exit-on-host-close".into()],
            true,
        );
        assert_eq!(shell, None);
    }

    #[test]
    fn scopes_language_server_path_check_away_from_third_party_ides() {
        assert!(is_antigravity_language_server_path(None));
        assert!(is_antigravity_language_server_path(Some(Path::new(
            r"C:\Users\Alice\AppData\Local\Programs\antigravity\resources\app\extensions\bin\language_server.exe"
        ))));
        assert!(is_antigravity_language_server_path(Some(Path::new(
            r"C:\Users\Alice\.gemini\antigravity\bin\language_server.exe"
        ))));
        assert!(!is_antigravity_language_server_path(Some(Path::new(
            r"C:\Users\Alice\AppData\Local\Programs\Windsurf\resources\app\extensions\windsurf\bin\language_server.exe"
        ))));
        assert!(!is_antigravity_language_server_path(Some(Path::new(
            r"C:\Go\bin\language_server.exe"
        ))));
    }

    #[test]
    fn migrates_launch_context_and_keeps_unchanged_file() {
        let dir = tempfile::tempdir().unwrap();
        let context = dir.path().join(LAUNCH_CONTEXT_FILE);
        let legacy = serde_json::json!({
            "appArgs": ["--lang=zh-CN", "--unknown-flag"],
            "companion": {
                "executable": r"C:\Fixtures\agy-claude-style-watcher.exe",
                "args": ["--launch"],
                "spawnsApp": true
            }
        });
        fs::write(&context, legacy.to_string()).unwrap();

        let plan = load_launch_plan(dir.path());
        assert!(plan.companion.is_none());
        assert_eq!(plan.app_args, vec!["--lang=zh-CN"]);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&fs::read(&context).unwrap()).unwrap(),
            serde_json::json!({ "appArgs": ["--lang=zh-CN"] }),
        );

        let marker = std::time::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
        fs::OpenOptions::new()
            .write(true)
            .open(&context)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(marker))
            .unwrap();
        save_launch_args(dir.path(), &plan.app_args, None);
        assert_eq!(fs::metadata(&context).unwrap().modified().unwrap(), marker);
        save_launch_args(dir.path(), &["--disable-gpu".to_owned()], None);
        assert_ne!(fs::metadata(&context).unwrap().modified().unwrap(), marker);
        assert_eq!(load_launch_plan(dir.path()).app_args, vec!["--disable-gpu"]);
    }

    #[test]
    #[ignore = "requires an idle Antigravity window and saved work"]
    fn gracefully_exits_and_reopens_idle_antigravity() {
        assert_eq!(
            env::var("AGASL_TEST_GRACEFUL_EXIT").as_deref(),
            Ok("confirmed")
        );
        let dir = tempfile::tempdir().unwrap();
        let executable = installed().expect("Antigravity installation");
        assert_eq!(require_closed(), Err("client_running"));
        let plan = prepare_switch(&executable, dir.path()).expect("graceful exit");
        assert_eq!(require_closed(), Ok(()));
        open(&executable, &plan).expect("reopen the original Antigravity installation");
    }

    #[test]
    #[ignore = "requires Antigravity to be fully closed"]
    fn opens_closed_antigravity_without_inheriting_launcher_io() {
        assert_eq!(env::var("AGASL_TEST_OPEN").as_deref(), Ok("confirmed"));
        let executable = installed().expect("Antigravity installation");
        assert_eq!(require_closed(), Ok(()));
        open(&executable, &LaunchPlan::default())
            .expect("open the original Antigravity installation");
    }
}
