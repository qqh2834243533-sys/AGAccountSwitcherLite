use std::{
    ffi::OsString,
    os::windows::{
        ffi::OsStringExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    time::Instant,
};
use sysinfo::Process;
use windows_sys::Win32::{
    Foundation::{FILETIME, WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::Threading::{
        GetProcessTimes, OpenProcess, QueryFullProcessImageNameW, TerminateProcess,
        WaitForSingleObject, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
    },
};

pub(super) struct Target(OwnedHandle);

impl Target {
    pub(super) fn open(process: &Process) -> Result<Self, &'static str> {
        let expected_path = process.exe().ok_or("client_state_unknown")?;
        let handle = unsafe {
            OpenProcess(
                PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                0,
                process.pid().as_u32(),
            )
        };
        if handle.is_null() {
            return Err("close_failed");
        }
        let handle = unsafe { OwnedHandle::from_raw_handle(handle) };
        let mut path = vec![0; 32768];
        let mut len = path.len() as u32;
        let read_path = unsafe {
            QueryFullProcessImageNameW(
                handle.as_raw_handle(),
                PROCESS_NAME_WIN32,
                path.as_mut_ptr(),
                &mut len,
            )
        };
        if read_path == 0 {
            return Err("client_state_unknown");
        }
        let actual_path = OsString::from_wide(&path[..len as usize]);
        let [mut created, mut exited, mut kernel, mut user] = [FILETIME::default(); 4];
        if unsafe {
            GetProcessTimes(
                handle.as_raw_handle(),
                &mut created,
                &mut exited,
                &mut kernel,
                &mut user,
            )
        } == 0
        {
            return Err("client_state_unknown");
        }
        let ticks = (u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime);
        let started = (ticks / 10_000_000).checked_sub(11_644_473_600);
        if started != Some(process.start_time())
            || !actual_path
                .to_string_lossy()
                .eq_ignore_ascii_case(&expected_path.to_string_lossy())
        {
            return Err("client_state_changed");
        }
        Ok(Self(handle))
    }

    fn wait(&self, milliseconds: u32) -> Result<bool, &'static str> {
        match unsafe { WaitForSingleObject(self.0.as_raw_handle(), milliseconds) } {
            WAIT_OBJECT_0 => Ok(true),
            WAIT_TIMEOUT => Ok(false),
            _ => Err("close_failed"),
        }
    }
}

pub(super) fn remaining(targets: &[Target], deadline: Instant) -> Result<(), &'static str> {
    for target in targets {
        if target.wait(0)? {
            continue;
        }
        if unsafe { TerminateProcess(target.0.as_raw_handle(), 1) } == 0 && !target.wait(0)? {
            return Err("close_failed");
        }
    }
    for target in targets {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let milliseconds = remaining.as_millis().min(u128::from(u32::MAX - 1)) as u32;
        if !target.wait(milliseconds)? {
            return Err("close_timeout");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env,
        process::{Child, Command, Stdio},
        thread,
        time::Duration,
    };
    use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

    struct Fixture(Child);

    impl Fixture {
        fn start() -> Self {
            Self(
                Command::new(env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "--ignored",
                        "client::terminate::tests::wait_for_termination",
                    ])
                    .stdout(Stdio::null())
                    .spawn()
                    .unwrap(),
            )
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    #[test]
    #[ignore = "helper process for the Windows termination test"]
    fn wait_for_termination() {
        thread::sleep(Duration::from_secs(60));
    }

    #[test]
    fn terminates_only_targets_and_waits_for_windows_exit() {
        let mut target = Fixture::start();
        let mut other = Fixture::start();
        let pid = Pid::from_u32(target.0.id());
        let mut system = System::new();
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            ProcessRefreshKind::new().with_exe(UpdateKind::Always),
        );
        let handle = Target::open(system.process(pid).unwrap()).unwrap();
        remaining(&[handle], Instant::now() + Duration::from_secs(5)).unwrap();
        assert_eq!(target.0.try_wait().unwrap().unwrap().code(), Some(1));
        assert!(other.0.try_wait().unwrap().is_none());
    }

    #[test]
    fn skips_already_exited_targets_and_terminates_remaining() {
        let mut already_exited = Fixture::start();
        let mut still_running = Fixture::start();
        let pid1 = Pid::from_u32(already_exited.0.id());
        let pid2 = Pid::from_u32(still_running.0.id());
        let mut system = System::new();
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid1, pid2]),
            ProcessRefreshKind::new().with_exe(UpdateKind::Always),
        );
        let h1 = Target::open(system.process(pid1).unwrap()).unwrap();
        let h2 = Target::open(system.process(pid2).unwrap()).unwrap();

        let _ = already_exited.0.kill();
        let _ = already_exited.0.wait();

        remaining(&[h1, h2], Instant::now() + Duration::from_secs(5)).unwrap();
        assert_eq!(still_running.0.try_wait().unwrap().unwrap().code(), Some(1));
    }
}
