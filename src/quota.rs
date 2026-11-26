use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::File, io::Read, path::Path, time::SystemTime};

const CACHE_FILE: &str = "quota-cache.json";
const CACHE_VERSION: u8 = 3;
const CACHE_LIMIT: u64 = 256 * 1024;
const MAX_BUCKETS: usize = 48;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptureError {
    SourceSlotChanged,
    Unavailable,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Serialize)]
pub enum Window {
    #[serde(rename = "5h")]
    FiveHours,
    #[serde(rename = "weekly")]
    Weekly,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    captured_at: i64,
    buckets: Vec<Bucket>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Bucket {
    id: String,
    window: Window,
    remaining_fraction: Option<f64>,
    reset_at: Option<i64>,
}

#[derive(Deserialize)]
struct Summary {
    response: Groups,
}

#[derive(Deserialize)]
struct Groups {
    groups: Vec<Group>,
}

#[derive(Deserialize)]
struct Group {
    #[serde(default)]
    buckets: Vec<ReportedBucket>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReportedBucket {
    bucket_id: Option<String>,
    window: Option<String>,
    remaining_fraction: Option<f64>,
    reset_time: Option<String>,
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

impl Snapshot {
    pub fn from_summary(bytes: &[u8]) -> Option<Self> {
        let captured_at = i64::try_from(SystemTime::UNIX_EPOCH.elapsed().ok()?.as_millis()).ok()?;
        let summary: Summary = serde_json::from_slice(bytes).ok()?;
        let mut buckets = Vec::new();
        for item in summary
            .response
            .groups
            .into_iter()
            .flat_map(|group| group.buckets)
        {
            let Some(id) = item.bucket_id.filter(|id| valid_id(id)) else {
                continue;
            };
            let window = match item.window.as_deref() {
                Some("5h") => Window::FiveHours,
                Some("weekly") => Window::Weekly,
                _ => continue,
            };
            let remaining_fraction = item.remaining_fraction.filter(|v| (0.0..=1.0).contains(v));
            let reset_at = item
                .reset_time
                .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
                .map(|time| time.timestamp_millis())
                .filter(|time| *time > 0);
            if remaining_fraction.is_none() && reset_at.is_none() {
                continue;
            }
            if buckets
                .iter()
                .any(|bucket: &Bucket| bucket.id == id && bucket.window == window)
            {
                return None;
            }
            buckets.push(Bucket {
                id,
                window,
                remaining_fraction,
                reset_at,
            });
            if buckets.len() > MAX_BUCKETS {
                return None;
            }
        }
        (!buckets.is_empty()).then_some(Self {
            captured_at,
            buckets,
        })
    }

    fn valid(&self) -> bool {
        self.captured_at > 0
            && !self.buckets.is_empty()
            && self.buckets.len() <= MAX_BUCKETS
            && self.buckets.iter().all(|b| {
                valid_id(&b.id)
                    && b.remaining_fraction
                        .is_none_or(|v| (0.0..=1.0).contains(&v))
                    && b.reset_at.is_none_or(|time| time > 0)
            })
    }
}

#[derive(Deserialize, Serialize)]
pub struct Cache {
    version: u8,
    slots: BTreeMap<u8, SlotSnapshot>,
}

#[derive(Deserialize, Serialize)]
struct SlotSnapshot {
    binding: String,
    snapshot: Snapshot,
}

impl Cache {
    pub fn load(root: &Path) -> Self {
        fn read(root: &Path) -> Option<Cache> {
            let mut bytes = Vec::new();
            File::open(root.join(CACHE_FILE))
                .ok()?
                .take(CACHE_LIMIT + 1)
                .read_to_end(&mut bytes)
                .ok()?;
            if bytes.len() as u64 > CACHE_LIMIT {
                return None;
            }
            let mut cache: Cache = serde_json::from_slice(&bytes).ok()?;
            if cache.version != CACHE_VERSION {
                return None;
            }
            cache.slots.retain(|_, entry| {
                entry.binding.len() == 64
                    && entry.binding.bytes().all(|byte| byte.is_ascii_hexdigit())
                    && entry.snapshot.valid()
            });
            Some(cache)
        }
        read(root).unwrap_or_else(|| Self {
            version: CACHE_VERSION,
            slots: BTreeMap::new(),
        })
    }

    pub fn get(&self, slot: u8, binding: &str) -> Option<&Snapshot> {
        let entry = self.slots.get(&slot)?;
        (entry.binding == binding).then_some(&entry.snapshot)
    }

    pub fn store(
        root: &Path,
        slot: u8,
        binding: String,
        snapshot: Snapshot,
    ) -> std::io::Result<()> {
        let mut cache = Self::load(root);
        cache.slots.insert(slot, SlotSnapshot { binding, snapshot });
        crate::config_file::replace(&root.join(CACHE_FILE), &serde_json::to_vec(&cache)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_account_caches_are_ignored_and_new_entries_require_the_slot_binding() {
        let root = tempfile::tempdir().unwrap();
        let key = "a".repeat(64);
        let snapshot = Snapshot::from_summary(
            br#"{"response":{"groups":[{"buckets":[{"bucketId":"gemini-weekly","window":"weekly","remainingFraction":0.5}]}]}}"#,
        )
        .unwrap();
        for version in [1, 2] {
            let old = serde_json::json!({
                "version": version,
                "accounts": { (key.clone()): snapshot }
            });
            std::fs::write(
                root.path().join(CACHE_FILE),
                serde_json::to_vec(&old).unwrap(),
            )
            .unwrap();
            assert!(Cache::load(root.path()).get(1, &key).is_none());
        }

        Cache::store(root.path(), 1, key.clone(), snapshot).unwrap();
        let cache = Cache::load(root.path());
        assert!(cache.get(1, &key).is_some());
        assert!(cache.get(2, &key).is_none());
        assert!(cache.get(1, &"b".repeat(64)).is_none());
    }

    #[test]
    fn summary_keeps_independent_windows_and_missing_reset_times() {
        let snapshot = Snapshot::from_summary(
            br#"{"response":{"groups":[{"buckets":[
                {"bucketId":"gemini-5h","window":"5h","remainingFraction":0,"resetTime":"2026-11-26T12:16:22Z"},
                {"bucketId":"gemini-weekly","window":"weekly","remainingFraction":0.86,"resetTime":"2026-12-03T07:16:22Z"},
                {"bucketId":"3p-weekly","window":"weekly","remainingFraction":1},
                {"bucketId":"other","window":"daily","remainingFraction":1}
            ]}]}}"#,
        )
        .unwrap();
        let [short, week, third_party] = snapshot.buckets.as_slice() else {
            panic!("three supported buckets expected");
        };
        assert!(short.window == Window::FiveHours && week.window == Window::Weekly);
        assert_eq!(short.remaining_fraction, Some(0.0));
        assert!(short.reset_at < week.reset_at);
        assert_eq!(third_party.reset_at, None);
    }
}
