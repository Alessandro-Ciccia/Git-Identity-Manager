# Git Identity Manager

Git Identity Manager is a local-first desktop utility for managing multiple Git and GitHub identities on one computer.

Milestone 6 adds restart-safe directory rules backed by Git conditional includes. Every global Git configuration change is previewed, conflict-checked, backed up, limited to an app-managed block, and verified. Generated include files contain only the selected profile’s `user.name` and `user.email`; unrelated Git configuration and credentials are not exposed to the frontend.

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
