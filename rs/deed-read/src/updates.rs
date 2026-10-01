// SPDX-License-Identifier: MPL-2.0
//! The `(updates …)` repo-deed vocabulary.
//!
//! Normative text: `1-formats/deed/vocabulary/updates.adoc` in
//! `hyperpolymath/standards`; policy: `docs/DEPENDABOT-POLICY.adoc` (owner
//! ruling D269).
//!
//! # Fail closed
//!
//! The clause is how a repo turns automated updates *off*. A reader that
//! skipped an unknown term would turn a typo (`:enabeld #f`) into "updates on",
//! so every deviation is an [`UpdatesError`]: an unknown field or clause, a
//! duplicate, a wrongly typed value, a missing required field. The caller must
//! treat any error as "arm nothing for this repo, and report".
//!
//! No deed, or a deed with no `(updates …)` clause, is **not** an error: the
//! defaults ([`UpdatesPolicy::default`]) apply.

use std::fmt;
use std::path::Path;

use crate::syntax::{self, Node, Value};

/// Why a deed's `(updates …)` clause could not be read. Always fail closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatesError(pub String);

impl fmt::Display for UpdatesError {
    /// Render the reason, prefixed so a log line names the vocabulary.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(updates …): {}", self.0)
    }
}

impl std::error::Error for UpdatesError {}

/// Shorthand for building an [`UpdatesError`] result.
fn fail<T>(msg: impl Into<String>) -> Result<T, UpdatesError> {
    Err(UpdatesError(msg.into()))
}

/// A calendar date written `YYYY-MM-DD`. Ordering is chronological.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl Date {
    /// Parse exactly `YYYY-MM-DD`, rejecting impossible dates (`2026-02-30`).
    pub fn parse(s: &str) -> Result<Date, UpdatesError> {
        let b = s.as_bytes();
        let shape_ok = b.len() == 10
            && b[4] == b'-'
            && b[7] == b'-'
            && b.iter()
                .enumerate()
                .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit());
        if !shape_ok {
            return fail(format!(":until {s:?} is not YYYY-MM-DD"));
        }
        let year: u16 = s[0..4].parse().expect("four digits");
        let month: u8 = s[5..7].parse().expect("two digits");
        let day: u8 = s[8..10].parse().expect("two digits");
        let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
        let max_day = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => return fail(format!(":until {s:?} has no month {month}")),
        };
        if day == 0 || day > max_day {
            return fail(format!(":until {s:?} has no day {day} in month {month}"));
        }
        Ok(Date { year, month, day })
    }
}

impl fmt::Display for Date {
    /// Write the date back as `YYYY-MM-DD`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// `(hold …)`: keep one package below a version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hold {
    /// Dependabot `package-ecosystem`, `-` for `_` (`cargo`, `github-actions`).
    pub ecosystem: String,
    /// Exact package name; a trailing `*` makes it a prefix match.
    pub package: String,
    /// The first version that is NOT allowed.
    pub below: String,
    pub reason: String,
    /// `"#N"` or an issue URL.
    pub issue: Option<String>,
    /// After this date the hold has expired.
    pub until: Option<Date>,
}

impl Hold {
    /// Whether the hold still applies on `today` (inclusive of `:until`).
    pub fn is_active(&self, today: Date) -> bool {
        self.until.is_none_or(|u| today <= u)
    }

    /// Whether `name` is the held package (prefix match when it ends in `*`).
    pub fn matches_package(&self, name: &str) -> bool {
        match self.package.strip_suffix('*') {
            Some(prefix) => name.starts_with(prefix),
            None => name == self.package,
        }
    }
}

/// `(exclude …)`: no automated updates for one ecosystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exclude {
    pub ecosystem: String,
    pub reason: Option<String>,
}

/// A repo's update policy: the clause's contents, or the defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatesPolicy {
    pub enabled: bool,
    pub majors: bool,
    pub soak_days: u32,
    pub holds: Vec<Hold>,
    pub excludes: Vec<Exclude>,
}

impl Default for UpdatesPolicy {
    /// The policy for a repo with no deed or no `(updates …)` clause.
    fn default() -> Self {
        UpdatesPolicy {
            enabled: true,
            majors: true,
            soak_days: 0,
            holds: Vec::new(),
            excludes: Vec::new(),
        }
    }
}

impl UpdatesPolicy {
    /// Whether `ecosystem` is excluded in this repo.
    pub fn excludes_ecosystem(&self, ecosystem: &str) -> bool {
        self.excludes.iter().any(|e| e.ecosystem == ecosystem)
    }

    /// The holds that still apply on `today` to `package` in `ecosystem`.
    pub fn active_holds<'a>(
        &'a self,
        ecosystem: &'a str,
        package: &'a str,
        today: Date,
    ) -> impl Iterator<Item = &'a Hold> {
        self.holds.iter().filter(move |h| {
            h.ecosystem == ecosystem && h.matches_package(package) && h.is_active(today)
        })
    }
}

/// Read the policy from a parsed deed.
///
/// The clause may appear only in a `repo-deed`, at most once, as a direct child
/// of the form. Any deviation inside it is an error (see the module docs).
pub fn from_deed(doc: &Node) -> Result<UpdatesPolicy, UpdatesError> {
    let found: Vec<&Node> = doc.clauses_named("updates").collect();
    if found.is_empty() {
        return Ok(UpdatesPolicy::default());
    }
    if doc.head != "repo-deed" {
        return fail(format!("only a repo-deed may carry it, not a {}", doc.head));
    }
    if found.len() > 1 {
        return fail(format!("appears {} times; at most once is allowed", found.len()));
    }
    read_clause(found[0])
}

/// Parse deed source text and read its policy. A syntax error is an error.
pub fn from_text(text: &str) -> Result<UpdatesPolicy, UpdatesError> {
    let doc = syntax::parse(text).map_err(|e| UpdatesError(format!("deed does not parse: {e:#}")))?;
    from_deed(&doc)
}

/// Read the policy from a deed file. A missing file means the defaults; any
/// other I/O failure is an error, so an unreadable deed never means "on".
pub fn from_path(path: &Path) -> Result<UpdatesPolicy, UpdatesError> {
    match std::fs::read_to_string(path) {
        Ok(text) => from_text(&text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(UpdatesPolicy::default()),
        Err(e) => fail(format!("cannot read {}: {e}", path.display())),
    }
}

/// Check that every field name in `node` is in `allowed` and none repeats.
fn check_fields(node: &Node, allowed: &[&str]) -> Result<(), UpdatesError> {
    for (i, (key, _)) in node.fields.iter().enumerate() {
        if !allowed.contains(&key.as_str()) {
            return fail(format!(
                "unknown field :{key} in ({}); allowed: {}",
                node.head,
                allowed.iter().map(|a| format!(":{a}")).collect::<Vec<_>>().join(" ")
            ));
        }
        if node.fields[..i].iter().any(|(k, _)| k == key) {
            return fail(format!("field :{key} repeated in ({})", node.head));
        }
    }
    Ok(())
}

/// The value of a required field, or an error naming it.
fn required<'a>(node: &'a Node, key: &str) -> Result<&'a Value, UpdatesError> {
    node.field(key)
        .ok_or_else(|| UpdatesError(format!("({}) requires :{key}", node.head)))
}

/// The grammar name of a value's type, for error messages.
fn kind_of(v: &Value) -> &'static str {
    match v {
        Value::Str(_) => "string",
        Value::Int(_) => "integer",
        Value::Sym(_) => "symbol",
        Value::Bool(_) => "boolean",
        Value::Uuid5(_) => "uuid5",
        Value::List(_) => "list",
        Value::Quoted(_) => "quoted",
    }
}

/// A field value that must be a boolean; no coercion from symbols or strings.
fn want_bool(node: &Node, key: &str, v: &Value) -> Result<bool, UpdatesError> {
    v.as_bool().ok_or_else(|| {
        UpdatesError(format!("({}) :{key} must be #t or #f, got a {}", node.head, kind_of(v)))
    })
}

/// A field value that must be a string.
fn want_str(node: &Node, key: &str, v: &Value) -> Result<String, UpdatesError> {
    v.as_str().map(str::to_owned).ok_or_else(|| {
        UpdatesError(format!("({}) :{key} must be a string, got a {}", node.head, kind_of(v)))
    })
}

/// A field value that must be a symbol.
fn want_sym(node: &Node, key: &str, v: &Value) -> Result<String, UpdatesError> {
    v.as_sym().map(str::to_owned).ok_or_else(|| {
        UpdatesError(format!("({}) :{key} must be a symbol, got a {}", node.head, kind_of(v)))
    })
}

/// An optional string field.
fn opt_str(node: &Node, key: &str) -> Result<Option<String>, UpdatesError> {
    node.field(key).map(|v| want_str(node, key, v)).transpose()
}

/// Read the body of one `(updates …)` clause.
fn read_clause(node: &Node) -> Result<UpdatesPolicy, UpdatesError> {
    check_fields(node, &["enabled", "majors", "soak-days"])?;
    let mut policy = UpdatesPolicy::default();
    if let Some(v) = node.field("enabled") {
        policy.enabled = want_bool(node, "enabled", v)?;
    }
    if let Some(v) = node.field("majors") {
        policy.majors = want_bool(node, "majors", v)?;
    }
    if let Some(v) = node.field("soak-days") {
        let n = v.as_int().ok_or_else(|| {
            UpdatesError(format!("(updates) :soak-days must be an integer, got a {}", kind_of(v)))
        })?;
        policy.soak_days = u32::try_from(n)
            .map_err(|_| UpdatesError(format!("(updates) :soak-days must be 0 or more, got {n}")))?;
    }
    for child in &node.clauses {
        match child.head.as_str() {
            "hold" => policy.holds.push(read_hold(child)?),
            "exclude" => policy.excludes.push(read_exclude(child)?),
            other => return fail(format!("unknown clause ({other}); allowed: (hold …) (exclude …)")),
        }
    }
    Ok(policy)
}

/// Read one `(hold …)` clause.
fn read_hold(node: &Node) -> Result<Hold, UpdatesError> {
    check_fields(node, &["ecosystem", "package", "below", "reason", "issue", "until"])?;
    if !node.clauses.is_empty() {
        return fail("(hold) takes fields only, no nested clauses");
    }
    let reason = want_str(node, "reason", required(node, "reason")?)?;
    if reason.trim().is_empty() {
        return fail("(hold) :reason must not be empty");
    }
    let package = want_str(node, "package", required(node, "package")?)?;
    if package.is_empty() || package == "*" {
        return fail(format!("(hold) :package {package:?} names no package"));
    }
    let below = want_str(node, "below", required(node, "below")?)?;
    if below.is_empty() {
        return fail("(hold) :below must not be empty");
    }
    let until = opt_str(node, "until")?.map(|s| Date::parse(&s)).transpose()?;
    Ok(Hold {
        ecosystem: want_sym(node, "ecosystem", required(node, "ecosystem")?)?,
        package,
        below,
        reason,
        issue: opt_str(node, "issue")?,
        until,
    })
}

/// Read one `(exclude …)` clause.
fn read_exclude(node: &Node) -> Result<Exclude, UpdatesError> {
    check_fields(node, &["ecosystem", "reason"])?;
    if !node.clauses.is_empty() {
        return fail("(exclude) takes fields only, no nested clauses");
    }
    Ok(Exclude {
        ecosystem: want_sym(node, "ecosystem", required(node, "ecosystem")?)?,
        reason: opt_str(node, "reason")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wrap a clause body in a minimal valid repo-deed.
    fn deed(body: &str) -> String {
        format!(";; SPDX-License-Identifier: MPL-2.0\n(repo-deed :schema-version \"1.0.0\"\n{body})\n")
    }

    /// Read a clause body, expecting success.
    fn ok(body: &str) -> UpdatesPolicy {
        from_text(&deed(body)).unwrap_or_else(|e| panic!("expected ok, got {e}"))
    }

    /// Read a clause body, expecting failure; return the message.
    fn err(body: &str) -> String {
        match from_text(&deed(body)) {
            Ok(p) => panic!("expected an error, got {p:?}"),
            Err(e) => e.to_string(),
        }
    }

    /// No clause means defaults.
    #[test]
    fn no_clause_means_defaults() {
        assert_eq!(ok(""), UpdatesPolicy::default());
        assert!(UpdatesPolicy::default().enabled && UpdatesPolicy::default().majors);
    }

    /// Missing file means defaults but bad file does not.
    #[test]
    fn missing_file_means_defaults_but_bad_file_does_not() {
        let missing = Path::new("/nonexistent/deed-read/none_chora.deed");
        assert_eq!(from_path(missing).unwrap(), UpdatesPolicy::default());
        assert!(from_text("(repo-deed").is_err());
    }

    /// Full clause reads.
    #[test]
    fn full_clause_reads() {
        let p = ok(r##"(updates :enabled #t :majors #f :soak-days 7
            (hold :ecosystem cargo :package "tokio" :below "2.0.0"
              :reason "rework" :issue "#42" :until "2026-12-01")
            (exclude :ecosystem npm))"##);
        assert!(p.enabled && !p.majors);
        assert_eq!(p.soak_days, 7);
        assert_eq!(p.holds.len(), 1);
        assert_eq!(p.holds[0].until, Some(Date { year: 2026, month: 12, day: 1 }));
        assert!(p.excludes_ecosystem("npm") && !p.excludes_ecosystem("cargo"));
    }

    /// Opt out is read.
    #[test]
    fn opt_out_is_read() {
        assert!(!ok("(updates :enabled #f)").enabled);
    }

    // One mutant per reader obligation. Each must fail, and for its own reason.

    /// Typo field fails closed.
    #[test]
    fn typo_field_fails_closed() {
        assert!(err("(updates :enabeld #f)").contains("unknown field :enabeld"));
    }

    /// Unknown clause fails closed.
    #[test]
    fn unknown_clause_fails_closed() {
        assert!(err("(updates (holds :ecosystem cargo))").contains("unknown clause (holds)"));
    }

    /// Duplicate field fails closed.
    #[test]
    fn duplicate_field_fails_closed() {
        assert!(err("(updates :enabled #t :enabled #f)").contains("repeated"));
    }

    /// Duplicate clause fails closed.
    #[test]
    fn duplicate_clause_fails_closed() {
        assert!(err("(updates :enabled #t)\n(updates :enabled #f)").contains("at most once"));
    }

    /// No coercion.
    #[test]
    fn no_coercion() {
        assert!(err(r#"(updates :enabled "no")"#).contains("must be #t or #f"));
        assert!(err("(updates :majors off)").contains("must be #t or #f"));
        assert!(err(r#"(updates :soak-days "7")"#).contains("must be an integer"));
        assert!(err("(updates :soak-days -1)").contains("0 or more"));
        assert!(err(r#"(updates (exclude :ecosystem "npm"))"#).contains("must be a symbol"));
    }

    /// Hold without reason is rejected.
    #[test]
    fn hold_without_reason_is_rejected() {
        assert!(err(r#"(updates (hold :ecosystem cargo :package "x" :below "2"))"#)
            .contains("requires :reason"));
        assert!(err(r#"(updates (hold :ecosystem cargo :package "x" :below "2" :reason " "))"#)
            .contains("must not be empty"));
    }

    /// Hold requires package and below.
    #[test]
    fn hold_requires_package_and_below() {
        assert!(err(r#"(updates (hold :ecosystem cargo :below "2" :reason "r"))"#)
            .contains("requires :package"));
        assert!(err(r#"(updates (hold :ecosystem cargo :package "*" :below "2" :reason "r"))"#)
            .contains("names no package"));
        assert!(err(r#"(updates (hold :ecosystem cargo :package "x" :reason "r"))"#)
            .contains("requires :below"));
    }

    /// Until must be a real date.
    #[test]
    fn until_must_be_a_real_date() {
        let h = |d: &str| {
            format!(r#"(updates (hold :ecosystem cargo :package "x" :below "2" :reason "r" :until "{d}"))"#)
        };
        assert!(err(&h("2026-12-1")).contains("not YYYY-MM-DD"));
        assert!(err(&h("2026-13-01")).contains("no month 13"));
        assert!(err(&h("2026-02-29")).contains("no day 29"));
        assert_eq!(ok(&h("2028-02-29")).holds[0].until.unwrap().to_string(), "2028-02-29");
    }

    /// Only a repo deed may carry it.
    #[test]
    fn only_a_repo_deed_may_carry_it() {
        let text = ";; SPDX-License-Identifier: MPL-2.0\n\
                    (estate-deed :schema-version \"1.0.0\" (updates :enabled #f))\n";
        assert!(from_text(text).unwrap_err().to_string().contains("only a repo-deed"));
    }

    /// Holds match prefix and expire.
    #[test]
    fn holds_match_prefix_and_expire() {
        let p = ok(r#"(updates (hold :ecosystem github-actions :package "github/codeql-action*"
            :below "4.38.1" :reason "poisoned" :until "2026-12-01"))"#);
        let before = Date::parse("2026-12-01").unwrap();
        let after = Date::parse("2026-12-02").unwrap();
        assert_eq!(p.active_holds("github-actions", "github/codeql-action/init", before).count(), 1);
        assert_eq!(p.active_holds("github-actions", "github/codeql-action/init", after).count(), 0);
        assert_eq!(p.active_holds("cargo", "github/codeql-action", before).count(), 0);
        assert_eq!(p.active_holds("github-actions", "github/other", before).count(), 0);
    }
}
