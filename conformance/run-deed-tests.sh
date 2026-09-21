#!/usr/bin/env bash
# SPDX-FileCopyrightText: © 2026 Jonathan D.A. Jewell (hyperpolymath)
# SPDX-License-Identifier: MPL-2.0
#
# Behavioural test for .deed support in validate-action/validate-deed.sh.
#
# This exists because the gap it guards was invisible to every source-level
# survey. The discovery glob matched only '*.a2ml', so .deed files were never
# opened at all: the validator reported "no errors" and exited 0 having
# validated nothing. String presence of a regex is not behavioural acceptance —
# only running it settles it. So this runs the validator, and it asserts on the
# discovery COUNT, because "exit 0" is exactly what the bug produced.
#
# The four valid fixtures are one per ruled deed head (DEED-GRAMMAR-SPEC
# <<document-forms>>): estate-deed, repo-deed, praxis-deed, estate-atlas-deed.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VALIDATOR="${HERE}/../validate-action/validate-deed.sh"
FAILURES=0

# Report a passing assertion using the test script's standard output format.
ok()   { echo "  PASS: $1"; }
# Report a failing assertion and add it to the final failure count.
fail() { echo "  FAIL: $1"; FAILURES=$((FAILURES + 1)); }

# The conformance directories hold .a2ml fixtures too. Isolate the .deed ones
# so the discovery count is exact rather than incidental.
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/valid" "$WORK/invalid"
cp "$HERE"/valid/*.deed   "$WORK/valid/"
cp "$HERE"/invalid/*.deed "$WORK/invalid/"

echo "1. valid deeds — all four ruled heads, non-strict"
out="$(INPUT_PATH="$WORK/valid" bash "$VALIDATOR" 2>&1)"; rc=$?
grep -q 'Found 4 ' <<<"$out" && ok "discovered 4 deed files" \
    || fail "discovery: '$(grep -o 'Found [0-9]* [^ ]* file(s)' <<<"$out")' (expected 4)"
[[ $rc -eq 0 ]] && ok "exit 0" || fail "exit $rc (expected 0)"
grep -q '::error' <<<"$out" && fail "unexpected error annotation" || ok "no errors"

echo "2. valid deeds — strict"
out="$(INPUT_PATH="$WORK/valid" INPUT_STRICT=true bash "$VALIDATOR" 2>&1)"; rc=$?
[[ $rc -eq 0 ]] && ok "exit 0 under strict" || fail "exit $rc under strict (expected 0)"

# Severity is part of the contract, not decoration: identity is an ::error,
# version is a ::warning until strict promotes it. Asserting only that the
# filename appears somewhere let EITHER severity pass, so a regression that
# downgraded identity to a warning — or upgraded version to an error — would
# have gone unnoticed.
assert_ann() { # <severity> <fixture-regex> <message-substring>
    grep -qE "::$1 file=[^,]*$2,line=[0-9]+::.*$3" <<<"$out" \
        && ok "$2 -> ::$1 ($3)" || fail "$2: expected ::$1 matching '$3'"
}

echo "3. invalid deeds — non-strict: exact severities, and a non-zero exit"
out="$(INPUT_PATH="$WORK/invalid" bash "$VALIDATOR" 2>&1)"; rc=$?
grep -q 'Found 5 ' <<<"$out" && ok "discovered 5 deed files" \
    || fail "discovery: '$(grep -o 'Found [0-9]* [^ ]* file(s)' <<<"$out")' (expected 5)"
[[ $rc -ne 0 ]] && ok "non-zero exit (identity errors are present)" \
    || fail "exit 0 non-strict (expected non-zero — identity is an error)"

assert_ann error   'deed-missing-head\.deed'           'identity'
assert_ann warning 'deed-missing-version\.deed'        'version'

# The three below are negative controls for gate bypasses that were LIVE:
#   * deed-registry-version-only — `:registry-version` alone satisfied the
#     version gate, so an atlas head with no `:schema-version` passed.
#   * example-AI-MANIFEST.deed  — the `*AI-MANIFEST*` basename exemption was
#     not extension-guarded, so a .deed skipped BOTH identity and version.
#   * deed-head-not-first       — the head was matched on every line, so a file
#     could open with another form and buy identity from a later one.
# Each one exited 0 before the fix. They are here so that stays impossible.
assert_ann warning 'deed-registry-version-only\.deed'  'version'
assert_ann error   'deed-head-not-first\.deed'         'identity'
assert_ann error   'example-AI-MANIFEST\.deed'         'identity'
assert_ann warning 'example-AI-MANIFEST\.deed'         'version'

echo "4. invalid deeds — strict promotes the version warning to an error"
out="$(INPUT_PATH="$WORK/invalid" INPUT_STRICT=true bash "$VALIDATOR" 2>&1)"; rc=$?
[[ $rc -ne 0 ]] && ok "non-zero exit under strict" || fail "exit 0 under strict (expected non-zero)"
grep -qE "::error file=[^,]*deed-missing-version\.deed" <<<"$out" \
    && ok "version warning promoted to ::error under strict" \
    || fail "deed-missing-version.deed not promoted to ::error under strict"
grep -q '::warning' <<<"$out" \
    && fail "::warning still emitted under strict" || ok "no ::warning survives strict"

echo
if [[ $FAILURES -eq 0 ]]; then
    echo "All deed validator tests passed."
else
    echo "${FAILURES} deed validator test(s) failed."
    exit 1
fi
