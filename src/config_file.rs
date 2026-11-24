use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

pub fn replace(path: &Path, contents: &[u8]) -> io::Result<()> {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    let mut temporary_name = path
        .file_name()
        .expect("configuration file name")
        .to_os_string();
    temporary_name.push(format!(
        ".{}.{}.tmp",
        std::process::id(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed),
    ));
    let temporary_path = path.with_file_name(temporary_name);
    let mut temporary = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary_path)?;
    let written = temporary
        .write_all(contents)
        .and_then(|()| temporary.sync_all());
    drop(temporary);
    let replaced = written.and_then(|()| fs::rename(&temporary_path, path));
    if replaced.is_err() {
        let _ = fs::remove_file(temporary_path);
    }
    replaced
}
