# Git Identity Manager

Git Identity Manager is a local-first desktop utility for managing multiple Git and GitHub identities on one computer.

Milestone 4 adds native repository selection, restart-safe repository registration, worktree-aware validation, and read-only inspection of remotes and effective Git identity sources. Remote credentials are redacted, Git configuration is not modified, and profile assignment remains reserved for Milestone 5.

## Stack

- Tauri 2
- Svelte 5 + SvelteKit
- TypeScript
- Tailwind CSS
- Rust
- pnpm

## Development

Prerequisites:

- Node.js
- pnpm
- Rust and Cargo
- Git
- A current GitHub CLI (`gh`) release with `gh auth status --json hosts` support

Install dependencies:

```sh
pnpm install
```

Run checks:

```sh
pnpm format:check
pnpm lint
pnpm check
pnpm test
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Run the app during development:

```sh
pnpm tauri dev
```

## Documentation

- [Product](docs/PRODUCT.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Security](docs/SECURITY.md)
- [Milestones](docs/MILESTONES.md)

## Security

This application must not store GitHub tokens, passwords, private SSH keys, or credential-helper secrets. The frontend must not receive arbitrary shell execution capability. See [docs/SECURITY.md](docs/SECURITY.md).

## License

MIT
