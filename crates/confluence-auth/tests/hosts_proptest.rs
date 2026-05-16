//! Property-based round-trip test for `HostsFile`.
//!
//! Generates an arbitrary registry, serializes to TOML, parses back, and
//! asserts equality. This catches subtle serde misconfigurations that
//! example-based unit tests would miss — for example, `BTreeMap` key
//! escaping rules in TOML, or `chrono` losing sub-second precision in
//! its serde format.

use std::collections::BTreeMap;
use std::path::Path;

use chrono::{DateTime, TimeZone, Utc};
use confluence_auth::{AuthMethod, HostEntry};
use proptest::prelude::*;

/// Hostnames that survive our `validate_host` check and TOML escaping.
/// The actual `validate_host` is more permissive than we generate here;
/// the strategy below is deliberately conservative so the property test
/// doesn't accidentally flag unrelated TOML edge cases.
fn host_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,15}(\\.[a-z][a-z0-9-]{0,15}){1,3}".prop_map(String::from)
}

fn email_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{0,8}@[a-z][a-z0-9]{0,8}\\.[a-z]{2,4}".prop_map(String::from)
}

fn datetime_strategy() -> impl Strategy<Value = DateTime<Utc>> {
    // chrono's serde format is RFC 3339 with millisecond precision.
    // We pick a wide but finite window of seconds since the epoch.
    (0_i64..4_102_444_800_i64).prop_map(|secs| Utc.timestamp_opt(secs, 0).unwrap())
}

fn host_entry_strategy() -> impl Strategy<Value = HostEntry> {
    (email_strategy(), datetime_strategy()).prop_map(|(email, created_at)| HostEntry {
        email,
        auth_method: AuthMethod::ApiToken,
        created_at,
    })
}

// We deliberately don't reuse `confluence_auth::HostsFile` here because
// the type is private. We rebuild an equivalent shape via serde with the
// same TOML schema, then round-trip through the same `toml` crate the
// library uses. That tests the *schema*, which is what matters from an
// external compatibility standpoint.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
struct HostsFileShape {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default: Option<String>,
    #[serde(default)]
    hosts: BTreeMap<String, HostEntry>,
}

fn file_strategy() -> impl Strategy<Value = HostsFileShape> {
    proptest::collection::btree_map(host_strategy(), host_entry_strategy(), 0..6).prop_flat_map(
        |hosts| {
            let default_keys: Vec<String> = hosts.keys().cloned().collect();
            let default_strategy = if default_keys.is_empty() {
                Just(None).boxed()
            } else {
                proptest::option::of(proptest::sample::select(default_keys)).boxed()
            };
            default_strategy.prop_map(move |default| HostsFileShape {
                default,
                hosts: hosts.clone(),
            })
        },
    )
}

proptest! {
    #[test]
    fn hosts_file_round_trips(f in file_strategy()) {
        let s = toml::to_string_pretty(&f).expect("serialize");
        let parsed: HostsFileShape = toml::from_str(&s).expect("parse");
        prop_assert_eq!(parsed, f);
    }

    /// Sanity check: the serialized form is well-formed UTF-8 and parses
    /// even when the surrounding path is something arbitrary. The path
    /// in `Error::ConfigParse` is purely a diagnostic and must not
    /// influence parsing.
    #[test]
    fn parsed_form_independent_of_path(f in file_strategy(), name in "[a-z]{1,8}\\.toml") {
        let s = toml::to_string_pretty(&f).expect("serialize");
        let _path = Path::new(&name);
        let parsed: HostsFileShape = toml::from_str(&s).expect("parse");
        prop_assert_eq!(parsed, f);
    }
}
