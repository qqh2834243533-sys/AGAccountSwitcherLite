use std::{
    env,
    ffi::OsStr,
    fs,
    os::windows::ffi::OsStrExt,
    path::{Component, Path, PathBuf, Prefix},
    ptr::{null, null_mut},
    slice,
};
use windows_sys::Win32::{
    Foundation::ERROR_SUCCESS,
    Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW},
    System::Registry::{RegGetValueW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ},
};

const APP_PATH: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\Antigravity.exe";
const UNINSTALL_KEY: &str =
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\121a0be4-63bd-531e-acf8-fc3924c7e984";

pub fn installed() -> Option<PathBuf> {
    [
        ("LOCALAPPDATA", r"Programs\antigravity\Antigravity.exe"),
        ("ProgramFiles", r"Antigravity\Antigravity.exe"),
    ]
    .into_iter()
    .filter_map(|(root, suffix)| env::var_os(root).map(|value| PathBuf::from(value).join(suffix)))
    .find(|path| path.is_file())
    .or_else(|| {
        let path = registry_string(HKEY_LOCAL_MACHINE, APP_PATH, None)?;
        registered_executable(Path::new(&path))
    })
    .or_else(|| {
        [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER]
            .into_iter()
            .find_map(installer_registered_app)
    })
}

fn wide(text: impl AsRef<OsStr>) -> Vec<u16> {
    text.as_ref().encode_wide().chain(Some(0)).collect()
}

fn registry_string(hive: HKEY, key: &str, name: Option<&str>) -> Option<String> {
    let subkey = wide(key);
    let name = name.map(wide);
    let name_ptr = name.as_ref().map_or(null(), |value| value.as_ptr());
    let mut value = vec![0u16; 32_768];
    let mut bytes = (value.len() * size_of::<u16>()) as u32;
    let result = unsafe {
        RegGetValueW(
            hive,
            subkey.as_ptr(),
            name_ptr,
            RRF_RT_REG_SZ,
            null_mut(),
            value.as_mut_ptr().cast(),
            &mut bytes,
        )
    };
    if result != ERROR_SUCCESS {
        return None;
    }
    let value = value.get(..bytes as usize / size_of::<u16>())?;
    String::from_utf16(value.strip_suffix(&[0])?).ok()
}

fn installer_registered_app(hive: HKEY) -> Option<PathBuf> {
    let name = registry_string(hive, UNINSTALL_KEY, Some("DisplayName"))?;
    let publisher = registry_string(hive, UNINSTALL_KEY, Some("Publisher"))?;
    if !(name == "Antigravity" || name.starts_with("Antigravity "))
        || !matches!(publisher.as_str(), "Google" | "Google LLC")
    {
        return None;
    }
    let icon = registry_string(hive, UNINSTALL_KEY, Some("DisplayIcon"))?;
    let executable = registered_executable(icon_path(&icon))?;
    let uninstaller = executable.parent()?.join("Uninstall Antigravity.exe");
    // Corroborate user-writable installer metadata with the expected install layout
    // and PE product metadata. Product metadata is not a signature.
    (uninstaller.is_file() && has_antigravity_version_info(&executable)).then_some(executable)
}

fn icon_path(icon: &str) -> &Path {
    let path = icon
        .rsplit_once(',')
        .filter(|(_, index)| index.trim().parse::<i32>().is_ok())
        .map_or(icon, |(path, _)| path)
        .trim();
    Path::new(
        path.strip_prefix('"')
            .and_then(|p| p.strip_suffix('"'))
            .unwrap_or(path),
    )
}

fn registered_executable(path: &Path) -> Option<PathBuf> {
    fn is_local_app(path: &Path) -> bool {
        path.is_absolute()
            && matches!(
                path.components().next(),
                Some(Component::Prefix(prefix))
                    if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_))
            )
            && path
                .file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case("Antigravity.exe"))
    }

    if !is_local_app(path) {
        return None;
    }
    let resolved = fs::canonicalize(path).ok()?;
    (is_local_app(&resolved) && resolved.is_file()).then_some(resolved)
}

fn version_value<'a>(version: &'a [u32], field: &str, words: bool) -> Option<&'a [u16]> {
    let field = wide(field);
    let mut data = null_mut();
    let mut size = 0;
    if unsafe {
        VerQueryValueW(
            version.as_ptr().cast(),
            field.as_ptr(),
            &mut data,
            &mut size,
        )
    } == 0
        || data.is_null()
    {
        return None;
    }
    let count = if words {
        size as usize
    } else {
        size as usize / size_of::<u16>()
    };
    // VerQueryValueW returns an aligned pointer into the version buffer. String
    // lengths are UTF-16 code units, while the translation table is sized in bytes.
    let offset = data.addr().checked_sub(version.as_ptr().addr())?;
    let available = size_of_val(version).checked_sub(offset)?;
    if !data.cast::<u16>().is_aligned() || count > available / size_of::<u16>() {
        return None;
    }
    Some(unsafe { slice::from_raw_parts(data.cast(), count) })
}

fn has_antigravity_version_info(executable: &Path) -> bool {
    let filename = wide(executable);
    let size = unsafe { GetFileVersionInfoSizeW(filename.as_ptr(), null_mut()) };
    if size == 0 || size > 1024 * 1024 {
        return false;
    }
    let mut version = vec![0u32; (size as usize).div_ceil(size_of::<u32>())];
    if unsafe { GetFileVersionInfoW(filename.as_ptr(), 0, size, version.as_mut_ptr().cast()) } == 0
    {
        return false;
    }
    version_value(&version, r"\VarFileInfo\Translation", false).is_some_and(|translations| {
        translations.as_chunks::<2>().0.iter().any(|pair| {
            let strings = format!(r"\StringFileInfo\{:04x}{:04x}", pair[0], pair[1]);
            let property = |name| {
                let value = version_value(&version, &format!(r"{strings}\{name}"), true)?;
                String::from_utf16(value.strip_suffix(&[0]).unwrap_or(value)).ok()
            };
            property("ProductName").as_deref() == Some("Antigravity")
                && matches!(
                    property("CompanyName").as_deref(),
                    Some("Google" | "Google LLC")
                )
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_executable_requires_absolute_path_and_expected_filename() {
        let dir = tempfile::tempdir().unwrap();
        let valid = dir.path().join("Antigravity.exe");
        let other = dir.path().join("Other.exe");
        fs::write(&valid, []).unwrap();
        fs::write(&other, []).unwrap();
        assert!(registered_executable(&valid).is_some());
        assert!(registered_executable(&other).is_none());
        assert!(registered_executable(Path::new("Antigravity.exe")).is_none());
        assert!(!has_antigravity_version_info(&valid));
        assert_eq!(
            icon_path(r#""D:\Fixture Apps\Antigravity.exe",0"#),
            Path::new(r"D:\Fixture Apps\Antigravity.exe")
        );
    }
}
