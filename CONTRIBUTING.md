# Contributing

Thanks for your interest in Git Identity Manager. This document covers how to set
up the project, the standards a change must meet, and how releases are made.

By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).

## Prerequisites

- **Node.js** ≥ 22 (the repo pins a version in [`.nvmrc`](.nvmrc)).
- **pnpm** (the version in the `packageManager` field of `package.json`;
  `corepack enable` will provide it).
- **Rust** stable (`rustfmt` and `clippy` components; pinned in
  [`rust-toolchain.toml`](rust-toolchain.toml)).
- **Git** and a current **GitHub CLI** (`gh`) with `gh auth status --json hosts`
  support.
- Platform libraries required by Tauri 2 — see the
  [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/).

## Setup

```sh
pnpm install
pnpm tauri dev
```

## Project layout

| Path                       | Purpose                                              |
| -------------------------- | ---------------------------------------------------- |
| `src/lib/domain`           | Pure domain types and calculations (unit-tested).    |
| `src/lib/ipc`              | Typed wrappers around Tauri `invoke`.                |
| `src/lib/components`       | Feature panels and the first-run welcome screen.     |
| `src/lib/components/ui`    | Shared presentational primitives.                    |
| `src/lib/state`            | Shared client state modules.                         |
| `src/routes`               | Page shell, routing, and sidebar navigation.         |
| `src-tauri/src/commands`   | Thin Tauri IPC entry points.                         |
| `src-tauri/src/services`   | Validation, parsing, process execution, persistence. |
| `src-tauri/src/process.rs` | The process-execution seam (mockable in tests).      |

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for how these fit together.

## Security rules (non-negotiable)

These follow from the [security model](docs/SECURITY.md). A change that breaks
one of them will not be merged.

- Treat the frontend as untrusted. Do not expose arbitrary shell or filesystem
  access over Tauri IPC. Tauri commands model narrow application operations.
- Execute programs with executable + argument arrays. Never build `sh -c`
  strings or concatenate user input into a command.
- Validate every input in Rust: usernames, hostnames, repository paths, profile
  IDs, Git names, and emails.
- Never call `gh auth token` or `gh auth status --show-token` for normal
  functionality.
- Never store or log passwords, GitHub tokens, private SSH keys, or
  credential-helper secrets.
- Do not modify global Git, SSH, or credential configuration without an explicit
  user action, a preview of the exact change, a backup, and post-write
  verification.
- Keep parsing and process execution out of Svelte components; the UI consumes
  typed DTOs, not raw CLI output.

## Before you open a pull request

Run the full suite locally — CI runs the same checks and a pull request will not
merge until they pass:

```sh
pnpm format:check
pnpm lint
pnpm check
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

`pnpm format` fixes most formatting problems.

### Testing expectations

- New domain or state logic needs unit tests.
- New component behavior needs a component test that queries by role, accessible
  name, label, or text — never by CSS class.
- New backend behavior needs Rust service tests. Git integration tests must use
  disposable temporary repositories with isolated configuration and must never
  touch the developer's real Git config.
- Do not weaken or skip tests to make a change pass. Do not add mocks to
  production code to satisfy a test.

## Commit and pull request conventions

- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)
  (`feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`).
- Keep pull requests focused. Describe the change, the reasoning, and how you
  verified it. Fill in the pull request template.
- Update `CHANGELOG.md` under `[Unreleased]` for user-visible changes.
- Update the relevant `docs/` page when behavior or architecture changes.

## Releasing

Releases are cut by a maintainer:

1. Bump the version in `package.json`, `src-tauri/tauri.conf.json`, and
   `src-tauri/Cargo.toml`, and move `CHANGELOG.md` `[Unreleased]` entries into a
   new dated `[x.y.z]` section.
2. Merge that to `main`.
3. Push a matching tag: `git tag vX.Y.Z && git push origin vX.Y.Z`.
4. The [Release workflow](.github/workflows/release.yml) builds macOS, Windows,
   and Linux bundles and creates a **draft** GitHub release with the artifacts
   attached.
5. A maintainer reviews the draft and publishes it manually.

### Signed builds (follow-up)

Release binaries are currently **unsigned**. macOS shows a Gatekeeper warning and
Windows shows a SmartScreen prompt. Signing is planned:

- **macOS:** an Apple Developer ID Application certificate plus notarization
  credentials, consumed by `tauri-action` via
  `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`,
  `APPLE_ID`, `APPLE_PASSWORD`, and `APPLE_TEAM_ID`.
- **Windows:** an Authenticode certificate, configured through the Tauri bundler
  `windows.certificateThumbprint` / `signCommand` settings.

Add the secrets to the repository, then wire them into
`.github/workflows/release.yml`.
