# Git Identity Manager

Git Identity Manager is a local-first desktop utility for managing multiple Git and GitHub identities on one computer.

The project is currently in Milestone 0: architecture and bootstrap. Product functionality starts in Milestone 1 with local dependency detection for `git` and `gh`.

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
- GitHub CLI (`gh`) for later milestones

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
