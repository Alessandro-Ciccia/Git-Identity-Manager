# Git Identity Manager

Git Identity Manager is a local-first desktop app for developers who use more
than one Git and GitHub identity on the same computer. It discovers the GitHub
accounts you have already authenticated with the GitHub CLI, lets you define
reusable Git identity profiles, shows the effective identity of any repository
and fixes mismatches, and can apply directory-based rules through Git conditional
includes — showing you the exact change before it is made.

It never stores GitHub tokens, passwords, private SSH keys, or credential-helper
secrets. Those stay with the GitHub CLI, Git, and your operating system.

## Screenshots

> _Screenshots pending — add captures of the Overview screen and a "what will
> change" preview dialog to `docs/screenshots/` and embed them here._

## Features

- Discover GitHub CLI accounts and see which one is active per host.
- Switch the active GitHub account, with read-back verification.
- Start the official GitHub browser login flow without handling credentials.
- Define reusable Git identity profiles (name, email, optional GitHub account).
- Inspect a repository's effective `user.name` / `user.email` and where each
  value comes from (system, global, local, worktree, conditional include).
- Assign an expected profile to a repository and detect Git-identity or
  GitHub-account mismatches.
- Preview and apply repository-local identity fixes, verified with rollback on
  failure.
- Create directory rules backed by Git conditional includes, written into a
  delimited app-managed block with backups and verification.
- Dark / light / system theme, keyboard-navigable UI, and a guided first run.

## Supported platforms

macOS, Windows, and Linux. Development happens primarily on macOS; path and
filesystem handling does not assume POSIX semantics.

## Prerequisites

To run the app:

- **Git**
- A current **GitHub CLI** (`gh`) with `gh auth status --json hosts` support

To build from source, additionally:

- **Node.js** ≥ 22 and **pnpm** (see [`.nvmrc`](.nvmrc) and the
  `packageManager` field in `package.json`)
- **Rust** stable with `rustfmt` and `clippy` (see
  [`rust-toolchain.toml`](rust-toolchain.toml))
- Platform libraries for Tauri 2 — see the
  [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/)

## Install

### From a release

Download the installer for your platform from the
[Releases](https://github.com/Alessandro-Ciccia/Git-Identity-Manager/releases)
page. Release binaries are currently unsigned, so macOS Gatekeeper and Windows
SmartScreen will warn on first launch.

### From source

```sh
pnpm install
pnpm tauri build
```

The bundled application is written to `src-tauri/target/release/bundle/`.

## Development

```sh
pnpm install
pnpm tauri dev
```

Checks:

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

## Architecture

The frontend is untrusted: it receives no arbitrary shell or filesystem
capability. Every product action is a narrow Tauri command backed by a testable
Rust service that validates input and runs `git` / `gh` with argument arrays —
never a shell string. The UI consumes typed DTOs and does not parse CLI output.
Application data is stored locally in versioned JSON files and contains no
secrets.

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the full breakdown and
[`docs/PRODUCT.md`](docs/PRODUCT.md) for product goals.

## Security

Git Identity Manager must not store or expose GitHub tokens, passwords, private
SSH keys, or credential-helper secrets, and must not modify global Git, SSH, or
credential configuration without an explicit, previewed, verified action. The
threat boundary is documented in [`docs/SECURITY.md`](docs/SECURITY.md). To
report a vulnerability, see [`SECURITY.md`](SECURITY.md).

## Contributing

Contributions are welcome. Start with [`CONTRIBUTING.md`](CONTRIBUTING.md) and
the [Code of Conduct](CODE_OF_CONDUCT.md). Changes to behavior or architecture
should update the relevant `docs/` page and `CHANGELOG.md`.

## License

[MIT](LICENSE).
