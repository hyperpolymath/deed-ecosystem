// SPDX-License-Identifier: MPL-2.0
//! The reader against this hub's own `conformance/` deeds, and the
//! `(updates …)` fixture read end to end.
//!
//! `conformance/` also holds legacy `.a2ml` files; only `.deed` files are read.
//! Those fixtures are counted by `conformance/run-deed-tests.sh` too, which is
//! why the updates fixture lives under `tests/fixtures/` and not there.

use std::path::{Path, PathBuf};

use deed_read::{parse, updates};

/// `conformance/<sub>/*.deed` in the hub, sorted.
fn hub(sub: &str) -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conformance").join(sub);
    let mut out: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()))
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "deed"))
        .collect();
    out.sort();
    out
}

/// Read a file or panic with its path.
fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Hub valid deeds parse.
#[test]
fn hub_valid_deeds_parse() {
    let files = hub("valid");
    // run-deed-tests.sh asserts exactly 4; a smaller set means the path broke.
    assert_eq!(files.len(), 4, "expected the 4 hub valid deeds, found {files:?}");
    for path in files {
        parse(&read(&path)).unwrap_or_else(|e| panic!("{} should parse: {e:#}", path.display()));
    }
}

/// Hub invalid deeds are rejected.
#[test]
fn hub_invalid_deeds_are_rejected() {
    let files = hub("invalid");
    assert_eq!(files.len(), 5, "expected the 5 hub invalid deeds, found {files:?}");
    for path in files {
        assert!(parse(&read(&path)).is_err(), "{} should be rejected", path.display());
    }
}

/// Updates fixture reads every term.
#[test]
fn updates_fixture_reads_every_term() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/deed/valid/updates-clause_chora.deed");
    let p = updates::from_path(&path).expect("fixture reads");
    assert!(p.enabled);
    assert!(!p.majors, "fixture sets :majors #f; reading the default means the field was dropped");
    assert_eq!(p.soak_days, 7);
    assert_eq!(p.holds.len(), 1);
    let h = &p.holds[0];
    assert_eq!((h.ecosystem.as_str(), h.package.as_str(), h.below.as_str()), ("cargo", "tokio", "2.0.0"));
    assert_eq!(h.issue.as_deref(), Some("#42"));
    assert_eq!(h.until.map(|d| d.to_string()).as_deref(), Some("2026-12-01"));
    assert!(p.excludes_ecosystem("npm"));
}

/// Hub deeds without the clause get defaults.
#[test]
fn hub_deeds_without_the_clause_get_defaults() {
    for path in hub("valid") {
        assert_eq!(
            updates::from_text(&read(&path)).expect("reads"),
            updates::UpdatesPolicy::default(),
            "{}",
            path.display()
        );
    }
}
