use crate::vault::{Credential, Vault};
use serde::Serialize;
use std::io;
use zeroize::Zeroizing;

pub const SLOTS: std::ops::RangeInclusive<u8> = 1..=4;
type Result<T> = std::result::Result<T, &'static str>;

#[derive(Serialize)]
pub struct Account {
    slot: u8,
    label: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    accounts: Vec<Account>,
    active_slot: Option<u8>,
    unregistered_active: bool,
    recovery: Option<&'static str>,
    pending_slot: Option<u8>,
}

struct Journal {
    backup: Credential,
    slot: u8,
    previous_present: bool,
    pending: bool,
}

impl Journal {
    fn read(vault: &Vault<'_>) -> Result<Option<Self>> {
        let Some(backup) = credential_io(vault.journal())? else {
            return Ok(None);
        };
        let (marker, slot) = backup.comment.split_once(':').ok_or("journal_invalid")?;
        let slot = slot.parse().map_err(|_| "journal_invalid")?;
        validate_slot(slot).map_err(|_| "journal_invalid")?;
        let (pending, previous_present) = match marker {
            "p1" => (true, true),
            "p0" => (true, false),
            "r1" => (false, true),
            "r0" => (false, false),
            _ => return Err("journal_invalid"),
        };
        Ok(Some(Self {
            backup,
            slot,
            previous_present,
            pending,
        }))
    }

    fn persist(&self, vault: &Vault<'_>) -> Result<()> {
        let marker = match (self.pending, self.previous_present) {
            (true, true) => "p1",
            (true, false) => "p0",
            (false, true) => "r1",
            (false, false) => "r0",
        };
        credential_io(vault.save_journal(&self.backup, &format!("{marker}:{}", self.slot)))
    }

    fn previous_matches(&self, current: Option<&Credential>) -> bool {
        match current {
            Some(current) => self.previous_present && self.backup.same_session(current),
            None => !self.previous_present,
        }
    }
}

fn credential_io<T>(result: io::Result<T>) -> Result<T> {
    result.map_err(|_| "vault_unavailable")
}

fn validate_slot(slot: u8) -> Result<()> {
    if SLOTS.contains(&slot) {
        Ok(())
    } else {
        Err("account_missing")
    }
}

pub fn status(vault: &Vault<'_>) -> Result<Status> {
    let active = credential_io(vault.active())?.filter(|cred| !cred.secret.is_empty());
    let mut accounts = Vec::new();
    let mut matched_slots = Vec::new();

    for slot in SLOTS {
        if let Some(saved) = credential_io(vault.slot(slot))? {
            if let Some(ref current) = active {
                if saved.same_account(current) {
                    matched_slots.push(slot);
                }
            }
            accounts.push(Account {
                slot,
                label: saved.comment,
            });
        }
    }

    let (active_slot, unregistered_active) = match (&active, matched_slots.as_slice()) {
        (Some(_), [slot]) => (Some(*slot), false),
        (Some(_), []) => (None, true),
        _ => (None, false),
    };

    let journal = Journal::read(vault)?;
    Ok(Status {
        accounts,
        active_slot,
        unregistered_active,
        recovery: journal
            .as_ref()
            .map(|entry| if entry.pending { "pending" } else { "ready" }),
        pending_slot: journal
            .filter(|entry| entry.pending)
            .map(|entry| entry.slot),
    })
}

pub fn register(vault: &Vault<'_>, label: String) -> Result<()> {
    if Journal::read(vault)?.is_some_and(|entry| entry.pending) {
        return Err("recovery_required");
    }
    let label = label.trim();
    if label.is_empty() || label.chars().count() > 48 || label.chars().any(char::is_control) {
        return Err("invalid_label");
    }
    let active = credential_io(vault.active())?.ok_or("official_login_required")?;
    if active.secret.is_empty() {
        return Err("official_login_required");
    }
    let mut available = None;
    for slot in SLOTS {
        match credential_io(vault.slot(slot))? {
            Some(saved) => {
                if saved.same_account(&active) || saved.comment.eq_ignore_ascii_case(label) {
                    return Err("already_registered");
                }
            }
            None if available.is_none() => available = Some(slot),
            None => {}
        }
    }
    let slot = available.ok_or("slots_full")?;
    credential_io(vault.save(slot, &active, label))
}

pub fn remove(vault: &Vault<'_>, slot: u8) -> Result<()> {
    validate_slot(slot)?;
    let journal = Journal::read(vault)?;
    if let Some(ref entry) = journal {
        if entry.pending {
            return Err("recovery_required");
        }
    }
    credential_io(vault.remove(slot))?;
    if journal.is_some_and(|entry| !entry.pending && entry.slot == slot) {
        credential_io(vault.clear_journal())?;
    }
    Ok(())
}

/// If the current active credential uniquely matches a saved slot by stable
/// account identity (e.g. JWT `sub` or `refresh_token`) and has fresher bytes
/// from an official background token refresh, write it back to that slot.
/// Never overwrites any slot when the active credential belongs to an unknown
/// external account or matches multiple slots ambiguously.
fn sync_refreshed_slot(vault: &Vault<'_>, active: &Credential) -> Result<()> {
    let mut matched: Option<(u8, Credential)> = None;
    for slot in SLOTS {
        if let Some(saved) = credential_io(vault.slot(slot))? {
            if saved.same_account(active) {
                if matched.is_some() {
                    return Ok(());
                }
                matched = Some((slot, saved));
            }
        }
    }
    if let Some((slot, saved)) = matched {
        if !saved.same_session(active) {
            credential_io(vault.save(slot, active, &saved.comment))?;
        }
    }
    Ok(())
}

pub fn switch<P>(
    vault: &Vault<'_>,
    slot: u8,
    prepare: impl FnOnce() -> Result<P>,
    closed: impl Fn() -> Result<()>,
    launch: impl FnOnce(P) -> Result<()>,
) -> Result<()> {
    validate_slot(slot)?;
    if Journal::read(vault)?.is_some_and(|entry| entry.pending) {
        return Err("recovery_required");
    }
    let target = credential_io(vault.slot(slot))?.ok_or("account_missing")?;
    if target.secret.is_empty() {
        return Err("official_login_required");
    }
    let plan = prepare()?;
    closed()?;
    let previous = credential_io(vault.active())?;
    if let Some(active) = previous.as_ref() {
        sync_refreshed_slot(vault, active)?;
    }
    if previous
        .as_ref()
        .is_some_and(|entry| entry.same_account(&target))
    {
        return launch(plan).map_err(|_| "open_failed_without_switch");
    }
    let previous_present = previous.is_some();
    let mut journal = Journal {
        backup: previous.unwrap_or_else(|| Credential {
            secret: Zeroizing::new(vec![0]),
            user: String::new(),
            comment: String::new(),
        }),
        slot,
        previous_present,
        pending: true,
    };
    journal.persist(vault)?;

    let switched = (|| {
        closed()?;
        let current = credential_io(vault.active())?;
        if !journal.previous_matches(current.as_ref()) {
            return Err("active_changed");
        }
        credential_io(vault.update_active(&target))?;
        if !credential_io(vault.active())?.is_some_and(|entry| entry.same_session(&target)) {
            return Err("switch_unverified");
        }
        launch(plan)
    })();

    if let Err(reason) = switched {
        if restore_session(vault, &journal, &closed).is_err() {
            return Err("recovery_required");
        }
        return Err(reason);
    }

    journal.pending = false;
    journal.persist(vault).map_err(|_| "recovery_required")?;
    Ok(())
}

fn restore_session(
    vault: &Vault<'_>,
    journal: &Journal,
    closed: impl Fn() -> Result<()>,
) -> Result<()> {
    closed()?;
    let current = credential_io(vault.active())?;
    if !journal.previous_matches(current.as_ref()) {
        let target = credential_io(vault.slot(journal.slot))?.ok_or("journal_invalid")?;
        let Some(ref active) = current else {
            return Err("active_changed");
        };
        if !active.same_account(&target) {
            return Err("active_changed");
        }
        // Preserve any background token refresh performed by Antigravity on the
        // target slot before rolling active credentials back to `journal.backup`.
        sync_refreshed_slot(vault, active)?;
        if journal.previous_present {
            credential_io(vault.update_active(&journal.backup))?;
        } else {
            credential_io(vault.clear_active())?;
        }
        if !journal.previous_matches(credential_io(vault.active())?.as_ref()) {
            return Err("restore_unverified");
        }
    }
    credential_io(vault.clear_journal())
}

pub fn restore(vault: &Vault<'_>, closed: impl Fn() -> Result<()>) -> Result<()> {
    let journal = Journal::read(vault)?.ok_or("no_recovery")?;
    restore_session(vault, &journal, closed)
}

pub fn discard_recovery(vault: &Vault<'_>, closed: impl Fn() -> Result<()>) -> Result<()> {
    let journal = Journal::read(vault)?.ok_or("no_recovery")?;
    if journal.pending {
        closed()?;
        let active = credential_io(vault.active())?;
        let target = credential_io(vault.slot(journal.slot))?.ok_or("journal_invalid")?;
        if !journal.previous_matches(active.as_ref())
            && active.is_some_and(|entry| entry.same_account(&target))
        {
            return Err("restore_before_discard");
        }
    }
    credential_io(vault.clear_journal())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Fixture {
        prefix: String,
        active: String,
    }

    impl Fixture {
        fn new() -> Self {
            static NEXT_ID: AtomicU64 = AtomicU64::new(0);
            let clock = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let prefix = format!("ag-switch-lite-test:{}:{clock}:{id}", std::process::id());
            Self {
                active: format!("{prefix}:active"),
                prefix,
            }
        }
        fn vault(&self) -> Vault<'_> {
            Vault::fixture(&self.prefix, &self.active)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let vault = self.vault();
            for slot in SLOTS {
                let _ = vault.remove(slot);
            }
            let _ = vault.clear_active();
            let _ = vault.clear_journal();
        }
    }

    fn fake_session(value: u8) -> Credential {
        Credential {
            secret: Zeroizing::new(vec![value; 128]),
            user: "fixture".into(),
            comment: String::new(),
        }
    }

    fn matches(vault: &Vault<'_>, value: u8) -> bool {
        vault
            .active()
            .unwrap()
            .unwrap()
            .same_session(&fake_session(value))
    }

    #[test]
    fn one_click_round_trip_and_explicit_restore_use_windows_vault() {
        let fixture = Fixture::new();
        let vault = fixture.vault();
        vault.update_active(&fake_session(1)).unwrap();
        register(&vault, "First".into()).unwrap();
        vault.update_active(&fake_session(2)).unwrap();
        register(&vault, "Second".into()).unwrap();
        switch(
            &vault,
            1,
            || Ok(()),
            || Ok(()),
            |()| {
                assert!(matches(&vault, 1));
                Ok(())
            },
        )
        .unwrap();
        switch(
            &vault,
            2,
            || Ok(()),
            || Ok(()),
            |()| {
                assert!(matches(&vault, 2));
                Ok(())
            },
        )
        .unwrap();
        restore(&vault, || Ok(())).unwrap();
        assert!(matches(&vault, 1));
        assert!(status(&vault).unwrap().recovery.is_none());
    }

    #[test]
    fn launch_failure_restores_previous_or_absent_credential() {
        let fixture = Fixture::new();
        let vault = fixture.vault();
        vault.save(1, &fake_session(1), "First").unwrap();
        assert_eq!(
            switch(&vault, 1, || Ok(()), || Ok(()), |()| Err("launch_failed")),
            Err("launch_failed")
        );
        assert!(vault.active().unwrap().is_none());
        vault.update_active(&fake_session(2)).unwrap();
        assert_eq!(
            switch(&vault, 1, || Ok(()), || Ok(()), |()| Err("launch_failed")),
            Err("launch_failed")
        );
        assert!(matches(&vault, 2));
        assert!(status(&vault).unwrap().recovery.is_none());
    }

    #[test]
    fn external_login_during_recovery_is_preserved() {
        let fixture = Fixture::new();
        let vault = fixture.vault();
        vault.save(1, &fake_session(1), "First").unwrap();
        vault.update_active(&fake_session(2)).unwrap();
        let result = switch(
            &vault,
            1,
            || Ok(()),
            || Ok(()),
            |()| {
                vault.update_active(&fake_session(3)).unwrap();
                Err("launch_failed")
            },
        );
        assert_eq!(result, Err("recovery_required"));
        assert_eq!(restore(&vault, || Ok(())), Err("active_changed"));
        assert!(matches(&vault, 3));
        assert_eq!(status(&vault).unwrap().pending_slot, Some(1));
        discard_recovery(&vault, || Ok(())).unwrap();
        assert!(matches(&vault, 3));
    }

    #[test]
    fn running_client_and_fifth_account_are_blocked() {
        let fixture = Fixture::new();
        let vault = fixture.vault();
        for slot in SLOTS {
            vault.update_active(&fake_session(slot)).unwrap();
            register(&vault, format!("Account {slot}")).unwrap();
        }
        vault.update_active(&fake_session(5)).unwrap();
        assert_eq!(register(&vault, "Fifth".into()), Err("slots_full"));
        assert_eq!(
            switch(
                &vault,
                1,
                || Ok(()),
                || Err("client_running"),
                |()| panic!("must not launch")
            ),
            Err("client_running")
        );
        assert!(matches(&vault, 5));
        assert!(status(&vault).unwrap().recovery.is_none());
    }

    #[test]
    fn unconfirmed_or_incomplete_app_exit_never_writes_credentials() {
        let fixture = Fixture::new();
        let vault = fixture.vault();
        vault.save(1, &fake_session(1), "First").unwrap();
        vault.update_active(&fake_session(2)).unwrap();
        for reason in ["client_state_changed", "close_failed", "close_timeout"] {
            assert_eq!(
                switch(
                    &vault,
                    1,
                    || Err::<(), _>(reason),
                    || panic!("must not proceed past exit preflight"),
                    |()| panic!("must not launch"),
                ),
                Err(reason)
            );
            assert!(matches(&vault, 2));
            assert!(status(&vault).unwrap().recovery.is_none());
        }
    }

    fn oauth_session(sub: Option<&str>, refresh: &str, access: &str) -> Credential {
        use base64::{engine::general_purpose, Engine as _};
        let id_token = sub.map(|subject| {
            let header = general_purpose::URL_SAFE_NO_PAD.encode(r#"{"alg":"RS256"}"#);
            let claims =
                general_purpose::URL_SAFE_NO_PAD.encode(format!(r#"{{"sub":"{subject}"}}"#));
            format!("{header}.{claims}.sig")
        });
        let payload = serde_json::json!({
            "token": {
                "access_token": access,
                "token_type": "Bearer",
                "refresh_token": refresh,
                "expiry": "2026-11-24T06:00:00Z"
            },
            "auth_method": "consumer",
            "id_token": id_token
        });
        Credential {
            secret: Zeroizing::new(payload.to_string().into_bytes()),
            user: "antigravity".into(),
            comment: String::new(),
        }
    }

    #[test]
    fn syncs_refreshed_token_back_to_matching_slot_without_polluting_other_slots() {
        let fixture = Fixture::new();
        let vault = fixture.vault();

        let acct_a_v1 = oauth_session(Some("sub-a"), "rt-a-1", "at-a-1");
        let acct_b_v1 = oauth_session(Some("sub-b"), "rt-b-1", "at-b-1");
        let acct_a_v2 = oauth_session(Some("sub-a"), "rt-a-2", "at-a-2");
        let acct_c_unknown = oauth_session(Some("sub-c"), "rt-c-1", "at-c-1");

        vault.update_active(&acct_a_v1).unwrap();
        register(&vault, "Work".into()).unwrap();
        vault.update_active(&acct_b_v1).unwrap();
        register(&vault, "Personal".into()).unwrap();

        // Simulate Antigravity refreshing Account A's token in the background.
        vault.update_active(&acct_a_v2).unwrap();
        // Duplicate registration of the same account after refresh must be rejected.
        assert_eq!(
            register(&vault, "Work Duplicate".into()),
            Err("already_registered")
        );

        // Switching from refreshed A -> B writes A's latest credential back into slot 1.
        switch(&vault, 2, || Ok(()), || Ok(()), |()| Ok(())).unwrap();
        let slot1 = vault.slot(1).unwrap().unwrap();
        assert_eq!(slot1.comment, "Work");
        assert!(slot1.same_session(&acct_a_v2));

        // External unknown account C must NOT overwrite any existing slot when switching to A.
        vault.update_active(&acct_c_unknown).unwrap();
        let s = status(&vault).unwrap();
        assert_eq!(s.active_slot, None);
        assert!(s.unregistered_active);

        switch(&vault, 1, || Ok(()), || Ok(()), |()| Ok(())).unwrap();
        assert!(vault.slot(1).unwrap().unwrap().same_session(&acct_a_v2));
        assert!(vault.slot(2).unwrap().unwrap().same_session(&acct_b_v1));
        assert!(vault.active().unwrap().unwrap().same_session(&acct_a_v2));
        assert_eq!(status(&vault).unwrap().active_slot, Some(1));
    }

    #[test]
    fn restore_succeeds_after_background_token_refresh_and_completed_slot_can_be_removed() {
        let fixture = Fixture::new();
        let vault = fixture.vault();

        let acct_a = oauth_session(Some("sub-a"), "rt-a-1", "at-a-1");
        let acct_b_v1 = oauth_session(Some("sub-b"), "rt-b-1", "at-b-1");
        let acct_b_v2 = oauth_session(Some("sub-b"), "rt-b-2", "at-b-2");

        vault.update_active(&acct_a).unwrap();
        register(&vault, "Account A".into()).unwrap();
        vault.update_active(&acct_b_v1).unwrap();
        register(&vault, "Account B".into()).unwrap();

        // Start from Account A and switch to Account B (slot 2).
        vault.update_active(&acct_a).unwrap();
        switch(&vault, 2, || Ok(()), || Ok(()), |()| Ok(())).unwrap();
        assert_eq!(status(&vault).unwrap().active_slot, Some(2));
        assert_eq!(status(&vault).unwrap().recovery, Some("ready"));

        // Antigravity refreshes Account B's token while running.
        vault.update_active(&acct_b_v2).unwrap();
        assert_eq!(status(&vault).unwrap().active_slot, Some(2));

        // Restoring previous session must succeed (not fail with `active_changed`)
        // AND preserve Account B's refreshed token in slot 2!
        restore(&vault, || Ok(())).unwrap();
        assert!(vault.active().unwrap().unwrap().same_session(&acct_a));
        assert!(vault.slot(2).unwrap().unwrap().same_session(&acct_b_v2));
        assert_eq!(status(&vault).unwrap().active_slot, Some(1));

        // Switch to slot 2 again, then verify removing completed slot 2 works without
        // requiring manual `discard_recovery` first, and clears stale recovery.
        switch(&vault, 2, || Ok(()), || Ok(()), |()| Ok(())).unwrap();
        assert_eq!(status(&vault).unwrap().recovery, Some("ready"));
        remove(&vault, 2).unwrap();
        assert!(vault.slot(2).unwrap().is_none());
        assert_eq!(status(&vault).unwrap().recovery, None);
    }
}
