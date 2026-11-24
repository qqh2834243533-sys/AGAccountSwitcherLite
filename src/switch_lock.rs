//! Exclusive file lock held across the entire credential operation.
//! Adapted from upstream `anglee0323/agy-switch` (`CC BY-NC-SA 4.0`).

use std::{
    fs::{File, OpenOptions},
    path::Path,
};

pub(crate) struct SwitchLock {
    _file: File,
}

impl SwitchLock {
    pub(crate) fn acquire(root: &Path) -> Result<Self, String> {
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            // Deny concurrent opens; Windows releases the handle on process exit.
            options.share_mode(0);
        }
        let file = options
            .open(root.join("account-switch.lock"))
            .map_err(|error| {
                #[cfg(windows)]
                if error.raw_os_error() == Some(32) {
                    return "another_account_switch_in_progress".into();
                }
                format!("Cannot acquire account-switch lock: {error}")
            })?;
        // Never unlink the lock file: removing it lets two processes lock different handles.
        Ok(Self { _file: file })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_concurrent_switch_and_releases_on_drop() {
        let temp = tempfile::tempdir().unwrap();
        let first = SwitchLock::acquire(temp.path()).unwrap();
        assert!(
            matches!(SwitchLock::acquire(temp.path()), Err(error) if error == "another_account_switch_in_progress")
        );
        drop(first);
        assert!(SwitchLock::acquire(temp.path()).is_ok());
    }

    #[test]
    fn rejects_a_separate_process() {
        let temp = tempfile::tempdir().unwrap();
        let first = SwitchLock::acquire(temp.path()).unwrap();
        let probe = |expected: &str| {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "switch_lock::tests::child_probe",
                    "--ignored",
                    "--nocapture",
                ])
                .env("AGY_LITE_LOCK_TEST_DIR", temp.path())
                .env("AGY_LITE_LOCK_TEST_EXPECT", expected)
                .output()
                .unwrap();
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                output.status.success(),
                "child_probe failed (stdout: {stdout}, stderr: {stderr})"
            );
            assert!(
                stdout.contains("AGASL_CHILD_LOCK_PROBE_VERIFIED") && stdout.contains("1 passed"),
                "child_probe did not actually execute! stdout:\n{stdout}\nstderr:\n{stderr}"
            );
        };
        probe("busy");
        drop(first);
        probe("free");
    }

    #[test]
    #[ignore = "helper invoked by rejects_a_separate_process"]
    fn child_probe() {
        let path = std::env::var_os("AGY_LITE_LOCK_TEST_DIR")
            .expect("AGY_LITE_LOCK_TEST_DIR must be provided by parent test");
        let expected = std::env::var("AGY_LITE_LOCK_TEST_EXPECT").unwrap();
        let result = SwitchLock::acquire(Path::new(&path));
        if expected == "busy" {
            assert!(
                matches!(result, Err(ref error) if error == "another_account_switch_in_progress"),
                "expected lock to be busy, got: {}",
                result.err().unwrap_or_else(|| "Ok".into())
            );
        } else {
            assert!(result.is_ok(), "expected lock to be free");
        }
        println!("AGASL_CHILD_LOCK_PROBE_VERIFIED");
    }
}
