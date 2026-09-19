# Contributing

Thank you for your interest in contributing! This repository is a
**coordination hub** and a satellite of `hyperpolymath/standards`: it
does not own the A2ML specification or governance — those are pinned
from `standards` — but it owns the conformance fixtures, the membership
manifest, and the cross-repo drift CI. Human-readable documentation
lives in the root; machine-readable metadata lives under
`.machine_readable/`.

## How to Contribute

We welcome contributions in many forms:

- **Conformance fixtures:** Add or refine cases under
  `conformance/valid/` and `conformance/invalid/` (keep
  `conformance/manifest.a2ml` in sync).

- **Membership:** Propose member additions/removals via
  `.machine_readable/6a2/ECOSYSTEM.a2ml`, kept in lockstep with
  `.gitmodules` and `scripts/check-membership.sh`.

- **Drift CI & tooling:** Improve `.github/workflows/anchor-drift.yml`
  or the helper `scripts/`.

- **Documentation:** Enhance the docs or the AI manifest.

- **Bug reports:** File clear, reproducible issues.

## Getting Started

1.  **Read the AI Manifest:** Start with `0-AI-MANIFEST.a2ml` to
    understand the repository structure.

2.  **Initialise members:** Run `scripts/init-submodules.sh` to populate
    `members/` (each member is pinned by commit).

3.  **Check membership:** Run `scripts/check-membership.sh` to verify
    the manifest, `.gitmodules`, and submodule declarations agree.

4.  **Validate conformance:** Run the A2ML validator
    (`hyperpolymath/a2ml-validate-action`) over `conformance/valid`
    (expect 0 errors) and over `conformance/invalid` with strict mode
    (every case must error).

## Development Workflow

### Branch Naming

    docs/short-description       # Documentation
    test/what-added              # Test additions
    feat/short-description       # New features
    fix/issue-number-description # Bug fixes
    refactor/what-changed        # Code improvements
    security/what-fixed          # Security fixes

### Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

    <type>(<scope>): <description>

    [optional body]

    [optional footer]

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `ci`, `chore`,
`security`

## Reporting Bugs

Before reporting: 1. Search existing issues 2. Check if it’s already
fixed in `main`

When reporting, include: - Clear, descriptive title - Environment
details (OS, versions, toolchain) - Steps to reproduce - Expected vs
actual behaviour

## Code of Conduct

All contributors are expected to adhere to our [Code of
Conduct](CODE_OF_CONDUCT.md).

## License

By contributing, you agree that your contributions will be licensed
under the same licenses as the project: code under MPL-2.0 and
documentation under CC-BY-SA-4.0 (see LICENSE and LICENSES/).
