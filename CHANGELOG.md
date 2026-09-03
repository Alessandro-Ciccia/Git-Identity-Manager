# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
On each release, the entries below move from `[Unreleased]` into a new dated
version section.

## [Unreleased]

### Added

- **Environment detection** — independent `git` and `gh` detection with version
  and health reporting; refresh without exposing shell input.
- **GitHub accounts** — discover accounts from `gh auth status --json hosts`,
  show the active account per host, switch between validated accounts with
  read-back verification, and launch the official browser login flow without
  handling credentials.
- **Profiles** — reusable local Git identities (label, name, email, optional
  GitHub account reference) with validation and restart-safe persistence; no
  secrets stored.
- **Repository inspection** — register repositories via a native folder picker,
  inspect remotes and the effective `user.name`/`user.email` with their config
  scope and origin, and tolerate missing or moved repositories.
- **Repository profile assignment** — assign an expected profile to a
  repository, detect Git identity and GitHub account mismatches, preview the
  exact local changes, apply only `user.name`/`user.email` locally, and verify
  the result with rollback on failure.
- **Directory rules** — path-based rules backed by Git conditional includes,
  written into a delimited app-managed block with backups, conflict detection,
  and post-write verification; generated identity files contain only name and
  email.
- **Experience** — dark/light/system theme stored locally, semantic design
  tokens, shared accessible UI primitives, keyboard-navigable sidebar and
  modals, a Settings section, and a guided first-run welcome screen.

### Project

- Public documentation, contribution and security policies, Continuous
  Integration, a cross-platform build matrix, and a tag-triggered draft-release
  workflow.

[Unreleased]: https://github.com/Alessandro-Ciccia/Git-Identity-Manager/commits/main
