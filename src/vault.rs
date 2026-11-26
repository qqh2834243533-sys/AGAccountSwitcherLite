use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io;
use std::{ptr, slice};
use windows_sys::Win32::Foundation::ERROR_NOT_FOUND;
use windows_sys::Win32::Security::Credentials::{
    CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_MAX_CREDENTIAL_BLOB_SIZE,
    CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
};
use zeroize::{Zeroize, Zeroizing};

// The Windows App credential and its generic local-machine persistence match
// official Antigravity / upstream `agy-switch` v4.10.18.
const APP_TARGET: &str = "gemini:antigravity";

pub struct Vault<'a> {
    prefix: &'a str,
    active_target: &'a str,
}

pub struct Credential {
    pub secret: Zeroizing<Vec<u8>>,
    pub user: String,
    pub comment: String,
}

struct SessionIdentity {
    claims: Option<JwtClaims>,
    refresh_token: Option<Zeroizing<String>>,
}

impl SessionIdentity {
    fn subject(&self) -> Option<&str> {
        self.claims.as_ref()?.sub.as_deref()
    }
}

#[derive(Default, Deserialize)]
struct StoredTokenField {
    #[serde(default)]
    refresh_token: Option<String>,
}

impl Drop for StoredTokenField {
    fn drop(&mut self) {
        if let Some(ref mut token) = self.refresh_token {
            token.zeroize();
        }
    }
}

#[derive(Default, Deserialize)]
struct StoredOAuthPayload {
    #[serde(default)]
    id_token: Option<String>,
    #[serde(default)]
    token: Option<StoredTokenField>,
}

impl Drop for StoredOAuthPayload {
    fn drop(&mut self) {
        if let Some(ref mut jwt) = self.id_token {
            jwt.zeroize();
        }
    }
}

#[derive(Default, Deserialize)]
struct JwtClaims {
    sub: Option<String>,
}

impl Drop for JwtClaims {
    fn drop(&mut self) {
        self.sub.zeroize();
    }
}

impl Credential {
    pub fn same_session(&self, other: &Self) -> bool {
        self.secret.as_slice() == other.secret.as_slice() && self.user == other.user
    }

    /// Match by Google subject when both are present, or by an unchanged refresh token.
    /// Falls back to exact byte equality when the payload is opaque or unknown.
    pub fn same_account(&self, other: &Self) -> bool {
        if self.same_session(other) {
            return true;
        }
        if self.user != other.user {
            return false;
        }
        let (Some(left), Some(right)) = (self.identity(), other.identity()) else {
            return false;
        };
        match (left.subject(), right.subject()) {
            (Some(left), Some(right)) => left == right,
            _ => left.refresh_token.is_some() && left.refresh_token == right.refresh_token,
        }
    }

    pub fn has_conflicting_subject(&self, other: &Self) -> bool {
        let left = self.identity();
        let right = other.identity();
        matches!(
            (left.as_ref().and_then(SessionIdentity::subject), right.as_ref().and_then(SessionIdentity::subject)),
            (Some(left), Some(right)) if left != right
        )
    }

    /// Reject a stale cache entry when a slot is loaded with different credentials.
    pub fn quota_binding(&self) -> String {
        let identity = self.identity();
        let mut hash = Sha256::new();
        hash.update(b"AGASL quota slot v3\0");
        hash.update((self.user.len() as u64).to_le_bytes());
        hash.update(self.user.as_bytes());
        if let Some(subject) = identity.as_ref().and_then(SessionIdentity::subject) {
            hash.update(b"subject\0");
            hash.update(subject.as_bytes());
        } else if let Some(refresh_token) =
            identity.as_ref().and_then(|id| id.refresh_token.as_ref())
        {
            hash.update(b"refresh\0");
            hash.update(refresh_token.as_bytes());
        } else {
            hash.update(b"opaque\0");
            hash.update(self.secret.as_slice());
        }
        format!("{:x}", hash.finalize())
    }

    fn identity(&self) -> Option<SessionIdentity> {
        let raw_text = std::str::from_utf8(self.secret.as_slice()).ok()?;
        let decoded_bytes: Zeroizing<Vec<u8>>;
        let json_text = if let Some(encoded) = raw_text.strip_prefix("go-keyring-base64:") {
            decoded_bytes = Zeroizing::new(general_purpose::STANDARD.decode(encoded.trim()).ok()?);
            std::str::from_utf8(decoded_bytes.as_slice()).ok()?
        } else {
            raw_text
        };

        let parsed: StoredOAuthPayload = serde_json::from_str(json_text).ok()?;
        let claims = parsed.id_token.as_deref().and_then(extract_jwt_claims);
        let refresh_token = parsed
            .token
            .as_ref()
            .and_then(|token| token.refresh_token.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| Zeroizing::new(value.to_owned()));
        Some(SessionIdentity {
            claims,
            refresh_token,
        })
    }
}

fn extract_jwt_claims(jwt: &str) -> Option<JwtClaims> {
    let mut parts = jwt.split('.');
    let _header = parts.next()?;
    let payload_b64 = parts.next()?;
    let _sig = parts.next()?;
    if parts.next().is_some() || payload_b64.is_empty() {
        return None;
    }
    let decoded = Zeroizing::new(
        general_purpose::URL_SAFE_NO_PAD
            .decode(payload_b64)
            .or_else(|_| general_purpose::URL_SAFE.decode(payload_b64))
            .ok()?,
    );
    let payload_text = std::str::from_utf8(decoded.as_slice()).ok()?;
    let mut claims: JwtClaims = serde_json::from_str(payload_text).ok()?;
    if let Some(subject) = claims.sub.as_mut() {
        let trimmed = subject.trim();
        if trimmed.is_empty() || trimmed.len() != subject.len() {
            let normalized = (!trimmed.is_empty()).then(|| trimmed.to_owned());
            subject.zeroize();
            claims.sub = normalized;
        }
    }
    Some(claims)
}

impl Vault<'static> {
    pub fn production() -> Self {
        Self {
            prefix: "ag-account-switcher-lite:v1",
            active_target: APP_TARGET,
        }
    }
}

impl<'a> Vault<'a> {
    #[cfg(test)]
    pub(crate) fn fixture(prefix: &'a str, active_target: &'a str) -> Self {
        Self {
            prefix,
            active_target,
        }
    }

    pub fn slot(&self, slot: u8) -> io::Result<Option<Credential>> {
        read(&self.slot_target(slot))
    }

    pub fn save(&self, slot: u8, credential: &Credential, label: &str) -> io::Result<()> {
        write(&self.slot_target(slot), credential, Some(label))
    }

    pub fn remove(&self, slot: u8) -> io::Result<()> {
        delete(&self.slot_target(slot))
    }

    pub fn active(&self) -> io::Result<Option<Credential>> {
        read(self.active_target)
    }

    pub fn update_active(&self, credential: &Credential) -> io::Result<()> {
        write(self.active_target, credential, None)
    }

    pub fn clear_active(&self) -> io::Result<()> {
        delete(self.active_target)
    }

    pub fn journal(&self) -> io::Result<Option<Credential>> {
        read(&format!("{}:recovery", self.prefix))
    }

    pub fn save_journal(&self, credential: &Credential, marker: &str) -> io::Result<()> {
        write(
            &format!("{}:recovery", self.prefix),
            credential,
            Some(marker),
        )
    }

    pub fn clear_journal(&self) -> io::Result<()> {
        delete(&format!("{}:recovery", self.prefix))
    }

    fn slot_target(&self, slot: u8) -> String {
        format!("{}:account:{slot}", self.prefix)
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

unsafe fn read_wide(value: *const u16) -> String {
    if value.is_null() {
        return String::new();
    }
    let mut len = 0;
    while unsafe { *value.add(len) } != 0 {
        len += 1;
    }
    String::from_utf16_lossy(unsafe { slice::from_raw_parts(value, len) })
}

struct OwnedCredential(*mut CREDENTIALW);

impl Drop for OwnedCredential {
    fn drop(&mut self) {
        unsafe { CredFree(self.0.cast()) }
    }
}

fn read(target: &str) -> io::Result<Option<Credential>> {
    let target = wide(target);
    let mut raw = ptr::null_mut();
    if unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut raw) } == 0 {
        let error = io::Error::last_os_error();
        return if error.raw_os_error() == Some(ERROR_NOT_FOUND as i32) {
            Ok(None)
        } else {
            Err(error)
        };
    }
    let owned = OwnedCredential(raw);
    let credential = unsafe { &*owned.0 };
    let secret = if credential.CredentialBlobSize == 0 {
        Vec::new()
    } else {
        unsafe {
            slice::from_raw_parts(
                credential.CredentialBlob,
                credential.CredentialBlobSize as usize,
            )
        }
        .to_vec()
    };
    Ok(Some(Credential {
        secret: Zeroizing::new(secret),
        user: unsafe { read_wide(credential.UserName) },
        comment: unsafe { read_wide(credential.Comment) },
    }))
}

fn write(target: &str, credential: &Credential, comment: Option<&str>) -> io::Result<()> {
    let blob_size = u32::try_from(credential.secret.len())
        .ok()
        .filter(|&size| size <= CRED_MAX_CREDENTIAL_BLOB_SIZE)
        .ok_or(io::ErrorKind::FileTooLarge)?;
    let mut target = wide(target);
    let mut user = wide(&credential.user);
    let mut comment_buf = comment.map(wide);
    let mut raw: CREDENTIALW = unsafe { std::mem::zeroed() };
    raw.Type = CRED_TYPE_GENERIC;
    raw.Persist = CRED_PERSIST_LOCAL_MACHINE;
    raw.TargetName = target.as_mut_ptr();
    raw.UserName = if user.len() == 1 {
        ptr::null_mut()
    } else {
        user.as_mut_ptr()
    };
    raw.Comment = match comment_buf.as_mut() {
        Some(buf) => buf.as_mut_ptr(),
        None => ptr::null_mut(),
    };
    raw.CredentialBlobSize = blob_size;
    raw.CredentialBlob = credential.secret.as_ptr().cast_mut();
    // CredWriteW atomically replaces an existing entry without a delete-first outage window.
    if unsafe { CredWriteW(&raw, 0) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn delete(target: &str) -> io::Result<()> {
    let target = wide(target);
    if unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) } == 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(ERROR_NOT_FOUND as i32) {
            return Err(error);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_cred(prefix_b64: bool, sub: Option<&str>, refresh: &str, access: &str) -> Credential {
        let id_token = sub.map(|subject| {
            let h = general_purpose::URL_SAFE_NO_PAD.encode(r#"{"alg":"RS256"}"#);
            let p = general_purpose::URL_SAFE_NO_PAD.encode(format!(r#"{{"sub":"{subject}"}}"#));
            format!("{h}.{p}.sig")
        });
        let raw_json = serde_json::json!({
            "token": {
                "access_token": access,
                "token_type": "Bearer",
                "refresh_token": refresh,
                "expiry": "2026-11-24T12:00:00Z"
            },
            "auth_method": "consumer",
            "id_token": id_token
        })
        .to_string();

        let encoded = if prefix_b64 {
            format!(
                "go-keyring-base64:{}",
                general_purpose::STANDARD.encode(raw_json.as_bytes())
            )
        } else {
            raw_json
        };

        Credential {
            secret: Zeroizing::new(encoded.into_bytes()),
            user: "antigravity".into(),
            comment: String::new(),
        }
    }

    #[test]
    fn matches_same_account_across_keyring_formats_and_token_refreshes() {
        let raw_v1 = build_cred(false, Some("google-uid-1"), "rt-1", "at-1");
        let b64_v2 = build_cred(true, Some("google-uid-1"), "rt-rotated", "at-2");
        let other = build_cred(false, Some("google-uid-2"), "rt-2", "at-1");

        assert!(!raw_v1.same_session(&b64_v2));
        assert!(raw_v1.same_account(&b64_v2));
        assert!(!raw_v1.same_account(&other));

        let rt_only_v1 = build_cred(false, None, "rt-stable", "at-1");
        let rt_only_v2 = build_cred(true, None, "rt-stable", "at-2");
        assert!(rt_only_v1.same_account(&rt_only_v2));
        assert_eq!(rt_only_v1.quota_binding(), rt_only_v2.quota_binding());

        let subject_and_rt = build_cred(false, Some("google-uid-1"), "rt-stable", "at-1");
        let conflicting_sub = build_cred(false, Some("google-uid-2"), "rt-stable", "at-1");
        let rotated_rt_only = build_cred(false, None, "rt-rotated", "at-2");
        assert!(subject_and_rt.same_account(&rt_only_v2));
        assert!(rt_only_v2.same_account(&subject_and_rt));
        assert!(!subject_and_rt.same_account(&conflicting_sub));
        assert!(!rt_only_v1.same_account(&rotated_rt_only));
        let same_subject_rotated = build_cred(true, Some("google-uid-1"), "rt-new", "at-2");
        assert_eq!(
            subject_and_rt.quota_binding(),
            same_subject_rotated.quota_binding()
        );
        assert_ne!(
            subject_and_rt.quota_binding(),
            conflicting_sub.quota_binding()
        );
        assert_ne!(rt_only_v1.quota_binding(), rotated_rt_only.quota_binding());
    }
}
