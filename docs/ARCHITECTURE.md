# Architecture

## Stack

- Tauri 2 for the desktop shell and native backend.
- Svelte 5 + SvelteKit in SPA/static mode for the UI.
- TypeScript with strict compiler options.
- Tailwind CSS for styling.
- Rust services for Git, GitHub CLI, persistence, and filesystem operations.
- pnpm for JavaScript package management.

## Trust boundary

The frontend is untrusted. It receives no unrestricted shell or filesystem
capability. Product actions are exposed as narrow Tauri commands with typed DTOs.

```text
UI components
→ frontend domain/application modules
→ typed Tauri command wrappers
→ Rust command handlers
→ Rust services
→ git / gh / local filesystem
```

CLI execution uses direct process execution with argument arrays. There is no
`sh -c` and no frontend-provided command string. Missing executables and per-tool
process failures are represented as status data rather than crashing or
preventing the other tool from being checked.

## Backend modules

Command modules are thin Tauri IPC entry points. Service modules own validation,
parsing, process execution, and persistence, and are independently testable.

- **`commands::environment` / `services::environment`** — independent `git` and
  `gh` detection, version parsing, and typed status DTOs.
- **`commands::github` / `services::github_cli`** — GitHub CLI account discovery,
  input validation, browser login, and switch-then-read-back verification.
- **`commands::profiles` / `services::profiles`** — profile normalization,
  authoritative validation, and versioned persistence with verified CRUD
  behavior.
- **`commands::repositories` / `services::repositories`** — repository
  registration keyed by a picked path or stored ID, with optional profile
  assignment and versioned local persistence.
- **`services::git`** — repository/worktree validation, remote parsing and
  credential redaction, effective identity and config-origin inspection, and
  fixed-key repository-local identity application with read-back verification and
  rollback.
- **`services::repository_assignment`** — assignment, unassignment, authoritative
  preview, local identity application, and verification orchestration across
  repository/profile storage and Git.
- **`commands::directory_rules` / `services::directory_rules`** — canonical
  path-rule persistence, app-owned identity include generation, managed global
  Git config block updates, backups, conflict detection, rollback, and
  verification.
- **`commands::preferences` / `services::preferences`** — versioned local
  persistence of the theme preference (`system`/`light`/`dark`) and the first-run
  dismissal flag. No Git, GitHub, SSH, repository, or credential access.
- **`process`** — the internal process-execution seam: direct
  executable/argument arrays, with fake adapters in tests. `SystemProcessRunner`
  appends the well-known `git`/`gh` install locations (Homebrew, MacPorts,
  `/usr/local`, `~/.local/bin`, …) to `PATH` when they exist and are missing, so
  detection and CLI calls work from a bundled app launched via Finder/Dock rather
  than a shell.

### GitHub account integration

Account discovery uses only `gh auth status --json hosts` and maps the result to
secret-free DTOs. Account mutations accept validated host/account identities
rather than command text. Switching executes a fixed `gh auth switch` argument
shape and returns success only after a second structured status read confirms the
selected account is active and healthy. Login starts GitHub CLI's non-interactive
browser flow without blocking Tauri IPC; the frontend polls structured status
until a new account appears while GitHub CLI retains the credential lifecycle.
`gh auth token` and `--show-token` are never used.

### Repository inspection and assignment

Repository selection uses Tauri's native dialog plugin with only the open-dialog
permission. The selected path is treated as untrusted input, canonicalized by
Rust, and verified to be a Git worktree; only the canonical top-level path is
stored. Inspection runs fixed read-only `git -C <path>` argument arrays for the
repository root, remotes, and the effective `user.name`/`user.email` with
scope/origin metadata, resolving linked worktree top levels and redacting URL
userinfo before returning typed DTOs. Registered repositories are re-inspected
independently, so a missing or moved path does not hide healthy registrations.
Reveal operations accept a registration ID, re-resolve its backend-owned stored
path, and never accept an arbitrary path to open.

Assigning a profile to a repository stores only a validated profile ID and
changes no Git configuration. Preview and apply commands accept a repository ID,
resolve the backend-owned registration and profile, and never accept paths,
config keys, command arguments, or desired identity values from the frontend.
Apply snapshots the existing local identity keys, writes only `user.name` and
`user.email` through fixed `git config --local` argument arrays, then verifies
both local and effective values. A failed write or verification attempts to
restore only those prior local values.

### Directory rules

Directory rules accept only a selected directory, an existing profile ID, and
optionally a backend-issued rule ID. Rust canonicalizes the directory and renders
`includeIf "gitdir:<directory>/"` entries into one clearly delimited app-managed
block. Generated include files contain only the profile's `user.name` and
`user.email` and live beneath Tauri's app configuration directory. Before apply,
the service detects overlapping app rules, potentially overlapping pre-existing
conditional includes, and registered repositories whose local/worktree identity
would override the rule. Existing files are backed up, writes use temporary
files, and failures restore the prior global config, generated identity file, and
rule store. Verification reads the generated file and asks Git to parse the
resulting conditional include.

## Frontend

- `src/lib/domain`: pure domain types and calculations — environment, GitHub
  account status, profile draft validation, repository assignment/mismatch
  evaluation, navigation, and theme-preference resolution.
- `src/lib/ipc`: typed, fixed-name wrappers around `invoke`.
- `src/lib/native`: the native directory picker wrapper.
- `src/lib/components`: feature panels (environment, GitHub accounts, profiles,
  repositories, directory rules, settings) plus the first-run welcome screen.
- `src/lib/components/ui`: shared presentational primitives — `Panel`, `Button`,
  `Badge`, `Banner`, `EmptyState`, and `Modal`. `Modal` owns the accessible
  dialog behavior: Escape-to-close (suppressed while an operation is in flight),
  focus trapping, focus-in on open, focus restoration on close, and body scroll
  lock.
- `src/lib/state`: shared client state modules (`theme.svelte.ts` resolves and
  applies the theme).
- `src/routes`: page shell, loading/error state, route-level composition, and
  roving-tabindex sidebar navigation.

The frontend consumes typed DTOs and never parses raw CLI output.

## Theming

Colors are semantic CSS custom properties defined in `src/app.css` and mapped
into Tailwind v4 utilities via `@theme inline`. Dark is the base palette on bare
`:root`; a matched light palette is applied under
`@media (prefers-color-scheme: light)` (for the `system` preference on a light
OS, with no JavaScript) and under `:root[data-theme='light']` for an explicit
choice, with `:root[data-theme='dark']` re-asserting dark. The backend preference
is authoritative; a light/dark override is also mirrored to `localStorage` and
applied in `src/routes/+layout.ts` before first paint so an explicit theme does
not flash on cold start. `system` writes no hint and no attribute.

## Persistence

Application data stays local and contains no authentication secrets. Each store
is a versioned JSON file beneath Tauri's application-data directory. Paths are
fixed by the backend, writes use a temporary file with backup recovery, and
successful mutations are read back before returning. Mutex-backed application
state serializes mutations.

- **`profiles.v1.json`** — profile ID, label, Git name, Git email, and an
  optional GitHub hostname/username reference. GitHub account health is read live
  and not persisted.
- **`repositories.v1.json`** — backend-generated ID, canonical worktree path,
  backend-generated timestamp, and optional profile ID. A missing/deleted profile
  reference is tolerated as an Unknown assignment state and cannot be applied.
- **`directory-rules.v1.json`** — backend-generated ID, canonical directory root,
  and profile ID. Generated identity files and durable pre-edit backups live
  below Tauri's app configuration directory.
- **`preferences.v1.json`** — theme preference and the first-run dismissal flag
  only. Additionally mirrored to browser `localStorage` as a non-authoritative
  paint-time hint.

The only user Git file modified is the existing user global config selected from
Git's standard home locations, or `~/.gitconfig` when neither standard file
exists. Existing symlinks are resolved so the target file is updated without
replacing the link.

## Testing strategy

- Frontend unit tests with Vitest for pure domain and state logic.
- Component tests where behavior matters, querying by role, accessible name,
  label, and text — never by CSS class.
- Rust unit tests for validation, parsing, command argument construction,
  profile/repository assignment persistence and restart behavior, mismatch
  preconditions, and service behavior.
- Git integration tests use disposable repositories with isolated global/system
  configuration and cover inherited/local/unset identities, conditional
  includes, linked worktrees, missing remotes, non-Git directories, verified
  local identity application, directory-rule effective identity, config
  preservation/backups/conflicts/removal, and rollback — without modifying the
  developer's Git configuration.
