# Security Model

This document describes how Git Identity Manager is designed to be safe. For
reporting a vulnerability, see [`SECURITY.md`](../SECURITY.md) at the repository
root.

## Principles

- Least privilege by default.
- The frontend is not trusted with shell or broad filesystem access.
- Secrets remain managed by GitHub CLI, Git, SSH, and the operating system
  credential store.
- Configuration-changing actions require clear user intent, preview,
  application, and verification.

## Prohibited behavior

The application must not:

- expose `execute(command: string)` or equivalent shell IPC;
- concatenate user input into shell commands;
- invoke `gh auth token` for normal functionality;
- invoke `gh auth status --show-token`;
- persist GitHub passwords, tokens, private SSH keys, or credential helper
  secrets;
- read private SSH key contents;
- rewrite `.gitconfig` or `.ssh/config` without explicit confirmation and backup
  where appropriate.

## GitHub CLI authentication

GitHub CLI is the credential owner:

- account discovery runs only `gh auth status --json hosts`;
- switching runs a fixed `gh auth switch --hostname <validated-host> --user <validated-user>`
  operation and verifies the result by reading status again;
- adding an account runs GitHub CLI's official web login flow and refreshes
  account state after completion;
- `gh auth token`, `--show-token`, token input, and human-readable auth-status
  parsing are not used;
- token source, scopes, raw CLI diagnostics, and credentials are not represented
  in frontend DTOs or logs;
- the GitHub CLI update action opens one fixed official installation URL through
  a narrow backend command; it cannot run a package manager or accept a
  frontend-provided URL.

## Profile persistence

Profile persistence is owned entirely by the Rust backend:

- the storage path is fixed beneath Tauri's application-data directory and is
  never accepted from the frontend;
- profile commands accept only typed profile fields or a profile ID;
- profile IDs, labels, Git names, Git emails, GitHub hostnames, and GitHub
  usernames are validated in Rust;
- the persisted GitHub association contains only hostname and username;
- tokens, passwords, private keys, credential-helper data, and raw GitHub CLI
  output are not represented in the profile schema;
- profile persistence does not read or modify Git, SSH, repository, or
  credential configuration.

## Repository inspection

Repository access is narrow and read-only:

- the frontend may open one native directory picker through `dialog:allow-open`,
  but receives no shell or general filesystem API;
- Rust treats the selected path as untrusted, canonicalizes it, verifies a Git
  worktree, and stores only the canonical top-level path;
- inspection executes fixed `git -C <validated-path>` argument arrays for the
  repository root, remotes, and the effective `user.name`/`user.email` with
  scope/origin metadata;
- config inspection does not list or return credential-helper values or
  unrelated Git configuration;
- URL userinfo is removed from remote URLs before DTO serialization so embedded
  passwords or tokens do not reach the frontend;
- reveal accepts only a validated registration ID and opens its revalidated
  backend-owned path;
- removing a registration only edits application data and never removes
  repository files.

## Repository profile assignment

Repository-local configuration changes are narrow and explicit:

- assigning, changing, or removing an expected profile edits only backend-owned
  application data and does not write Git configuration;
- mismatch evaluation consumes typed profile, repository inspection, and GitHub
  CLI account DTOs; it does not inspect credentials;
- preview/apply commands accept a repository registration ID, resolve its stored
  canonical path and assigned profile in Rust, and do not accept
  frontend-provided paths, config keys, command arguments, or identity values;
- preview is read-only and identifies current values/sources, desired values,
  and local scope before confirmation;
- apply executes only fixed direct `git config --local` operations for
  `user.name` and `user.email`;
- the prior local values for those two keys are captured for rollback, while
  unrelated local and all global/system configuration remain untouched;
- success requires immediate read-back of local and effective values; a write or
  verification failure produces a structured error and attempts to restore the
  two prior local values;
- stale profile references, missing repositories, invalid IDs, missing Git,
  command failures, and malformed output cannot be applied;
- GitHub account mismatch detection never invokes token-bearing commands and
  repository Fix does not switch accounts automatically.

## Directory rules

Global configuration changes are separate from repository-local Fix operations
and remain narrow and explicit:

- rule commands accept only a native-picker directory, existing profile ID, and
  optional backend-issued rule ID; they never accept config text, config keys,
  include targets, global config paths, or command arguments from the frontend;
- selected directories are validated, canonicalized, and converted by Rust to a
  recursive `gitdir:` conditional include;
- generated identity files contain only the selected profile's `user.name` and
  `user.email`; GitHub associations, tokens, signing keys, credential helpers,
  and unrelated profile data are excluded;
- the app owns one unmistakably delimited block in the user global Git config and
  preserves all bytes outside that block; malformed or duplicate block markers
  stop the operation;
- existing global config symlinks are resolved and their target is edited, rather
  than replacing the symlink;
- previews identify the directory condition, generated identity file, global
  config file, identity values, backup requirement, and conflicts without
  returning unrelated global config contents to the frontend;
- overlapping app rules, potentially overlapping existing conditional includes,
  and registered repository local/worktree identity overrides block apply;
- existing global and generated identity files are copied to app-owned backup
  storage before modification;
- writes use temporary files; apply and remove verify persisted metadata,
  generated identity content, and Git-parsed include entries, restoring prior
  config, identity, and store contents after a failure;
- removing a rule changes only the app-managed include block, its local rule
  metadata, and an app-generated identity file that is no longer shared by
  another rule;
- directory rules do not switch GitHub accounts, alter repository remotes, edit
  SSH configuration, or remove repository-local identity values.

## Application preferences

Preference storage is owned entirely by the Rust backend:

- the storage path is fixed beneath Tauri's application-data directory and is
  never accepted from the frontend;
- `set_theme_preference` accepts only the `system` / `light` / `dark` enum, and
  `set_welcome_dismissed` accepts only a boolean;
- the persisted file contains only the theme preference and the first-run
  dismissal flag — no tokens, paths, identities, or Git configuration;
- preference persistence does not read or modify Git, SSH, repository, or
  credential configuration;
- the theme value is mirrored to browser `localStorage` only as a
  non-authoritative paint-time hint; it carries no secret and the backend file
  remains the source of truth.

## Backend validation

Rust commands validate:

- usernames;
- hostnames;
- profile IDs;
- Git names and emails;
- repository paths;
- operation preconditions;
- assigned-profile existence and local identity verification;
- directory rule IDs, canonical directory roots, conditional-include conflicts,
  managed-block integrity, and post-write configuration state.

Detailed diagnostics are allowed for debugging, but user-facing errors are
structured and must not contain secrets.

## Tauri capabilities

Capabilities are minimal. The main window has the generated core baseline and
`dialog:allow-open`, used for repository and directory-rule folder selection.
Opener operations are narrow backend commands for fixed URLs or validated stored
repository paths. No frontend shell or general filesystem permission is granted.
Future changes must add permissions deliberately and document why they are
required.
