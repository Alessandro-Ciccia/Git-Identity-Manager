# Project: Git Identity Manager

You are the principal software engineer responsible for designing and implementing an open-source desktop application for managing multiple Git and GitHub identities.

The application is intended to solve a common developer problem: using multiple GitHub accounts and Git identities on the same computer without manually remembering Git, GitHub CLI, SSH, or repository-specific configuration commands.

The product should make this workflow understandable and safe even for developers who do not know the underlying Git commands.

Working name: **Git Identity Manager**.

The final name may be changed later.

---

# 1. Product vision

Build a polished cross-platform desktop application that allows developers to:

- discover GitHub accounts already authenticated through GitHub CLI;
- see which GitHub account is currently active;
- switch GitHub CLI account visually;
- define reusable Git identities/profiles;
- associate a Git identity with a GitHub account;
- configure `git user.name` and `git user.email`;
- inspect the identity currently used by a repository;
- assign a profile to a repository;
- detect identity mismatches;
- optionally configure automatic identity rules based on directories;
- support multiple SSH identities in a future phase;
- make all underlying changes transparent to the user.

The application must be open source and suitable for public distribution.

Do not build a proprietary backend or require a hosted service.

The application should work locally on the user's machine.

---

# 2. Technology stack

Use:

- Tauri 2
- Svelte 5
- TypeScript
- Tailwind CSS
- Rust for the privileged/native backend
- pnpm as package manager

Use strict TypeScript.

Prefer simple, maintainable dependencies.

Do not add large libraries unless there is a concrete reason.

Use current stable versions after checking the official documentation before installation.

---

# 3. Architectural principle

Treat the frontend as untrusted.

The frontend must NOT receive an unrestricted shell execution capability.

Do NOT implement something similar to:

```ts
execute(command: string)
```

or:

```ts
shell('whatever the frontend sends');
```

Instead expose narrow Tauri commands representing application operations.

Examples:

```text
get_environment_status()
list_github_accounts()
get_active_github_account()
switch_github_account(username)
launch_github_login()
inspect_git_repository(path)
get_git_identity(path)
set_repository_identity(path, profile)
get_global_git_identity()
set_global_git_identity(profile)
list_profiles()
create_profile()
update_profile()
delete_profile()
assign_repository_profile()
remove_repository_profile()
```

The Rust layer is responsible for validating every input and constructing CLI arguments.

Never concatenate untrusted strings into shell commands.

Prefer direct process execution with argument arrays rather than a shell.

---

# 4. External tools

The MVP may depend on:

- `git`
- `gh` / GitHub CLI

At startup detect:

```bash
git --version
gh --version
```

Show dependency status in the UI.

If one is unavailable, explain what is missing instead of crashing.

Do not automatically install system software without explicit user interaction.

---

# 5. GitHub account integration

Use GitHub CLI as the source of GitHub authentication for the MVP.

Use commands equivalent to:

```bash
gh auth status --json hosts
```

to discover authenticated GitHub accounts.

Switch account using:

```bash
gh auth switch --hostname github.com --user <username>
```

Adding an account should use the official browser login flow:

```bash
gh auth login --hostname github.com --web
```

Refresh account state after authentication completes.

IMPORTANT SECURITY REQUIREMENTS:

Never request or persist GitHub passwords.

Never persist GitHub access tokens.

Never expose tokens to the frontend.

Never invoke:

```bash
gh auth token
```

for normal application functionality.

Never invoke:

```bash
gh auth status --show-token
```

The credential lifecycle should remain managed by GitHub CLI / the operating system credential store.

Support GitHub Enterprise hosts in the internal domain model even if the initial UI primarily targets github.com.

---

# 6. Git identity management

A Git identity/profile should contain at minimum:

```ts
type GitProfile = {
  id: string;
  label: string;
  gitName: string;
  gitEmail: string;
  githubUsername?: string;
  githubHost?: string;
  sshIdentity?: string;
  signingKey?: string;
  signingEnabled?: boolean;
};
```

The exact model may evolve if architecture requires it.

Profiles are application concepts and should NOT contain authentication secrets.

Example profiles:

```text
Personal
GitHub: johndoe
Name: John Doe
Email: john@example.com

Work
GitHub: johndoe-company
Name: John Doe
Email: john@company.com
```

---

# 7. Repository inspection

Allow the user to select a repository using a native folder picker.

Validate that the selected folder belongs to a Git repository.

Inspect at least:

```bash
git -C <repo> rev-parse --show-toplevel
git -C <repo> remote -v
git -C <repo> config user.name
git -C <repo> config user.email
git -C <repo> config --show-origin --get user.name
git -C <repo> config --show-origin --get user.email
```

Prefer plumbing / structured commands where possible.

Normalize repository paths before storing them.

Handle Git worktrees correctly where practical.

---

# 8. Repository identity assignment

Allow assigning a profile to a repository.

For repository-specific configuration use:

```bash
git -C <repo> config --local user.name "<name>"
git -C <repo> config --local user.email "<email>"
```

Do not modify global Git configuration unless the user explicitly requests it.

Clearly display whether values currently originate from:

- system
- global
- local
- worktree
- conditional include

The UI should make precedence understandable.

---

# 9. Identity mismatch detection

This is one of the core features.

For each registered repository determine:

```text
Expected profile
Current Git identity
Current GitHub CLI account
Repository remote
Status
```

Possible statuses:

```text
Correct
Git identity mismatch
GitHub account mismatch
Both mismatch
Unknown
```

Example:

```text
Alpitour

Expected:
Work

Current Git identity:
personal@example.com

Current GitHub:
personal-account

⚠ Identity mismatch

[Fix]
```

The Fix action should show exactly what will be modified before performing the operation.

---

# 10. Profiles UI

Main application layout:

```text
Sidebar
├── Overview
├── Profiles
├── Repositories
├── Rules
└── Settings
```

Overview should immediately show:

```text
GitHub Accounts

Personal
@personal
ACTIVE

Work
@company
[Switch]
```

Then:

```text
Repositories

project-a     Personal     ✓
project-b     Work         ✓
project-c     Work         ⚠ Wrong identity
```

Avoid a typical "admin dashboard" visual style.

Aim for a modern native developer-tool aesthetic similar to:

- Linear
- Raycast
- GitHub Desktop
- modern VS Code tooling

without copying their designs.

Use:

- strong typography
- compact spacing
- subtle borders
- keyboard-friendly controls
- excellent dark mode
- clear status indicators
- minimal unnecessary decoration

The application should feel like a professional developer utility.

---

# 11. Repository registry

Allow users to register repositories with the application.

Persist metadata such as:

```ts
type RepositoryRegistration = {
  id: string;
  path: string;
  profileId?: string;
  addedAt: string;
};
```

The application must tolerate repositories being moved or deleted.

Show missing repositories without crashing.

Allow:

- add repository;
- remove repository from app;
- reveal/open repository;
- assign profile;
- change profile;
- inspect configuration.

Do not delete actual repository files.

---

# 12. Directory rules

After the repository-based MVP works, implement optional automatic directory rules.

Example:

```text
~/Developer/personal/** → Personal
~/Developer/work/**     → Work
```

Investigate and use Git conditional includes (`includeIf`) where appropriate.

Do NOT blindly rewrite `.gitconfig`.

Before modifying an existing Git configuration:

1. inspect the current file;
2. preserve unrelated configuration;
3. create a backup;
4. calculate the intended change;
5. show the user what will happen;
6. write safely;
7. verify the resulting Git configuration.

Generated identity files may use a structure similar to:

```text
~/.config/git-identity-manager/
├── personal.gitconfig
└── work.gitconfig
```

Do not assume this exact layout if a more standards-compliant location is preferable.

Research the correct cross-platform configuration directories first.

---

# 13. SSH — future architecture

Do not make SSH management a blocker for the initial MVP.

However, design the domain model so that a profile can later manage:

- SSH key;
- SSH alias;
- GitHub host;
- remote rewriting;
- SSH signing identity.

A future version may support configurations similar to:

```text
Host github-personal
    HostName github.com
    User git
    IdentityFile ~/.ssh/id_ed25519_personal

Host github-work
    HostName github.com
    User git
    IdentityFile ~/.ssh/id_ed25519_work
```

When this feature is implemented it must never overwrite unrelated SSH configuration.

---

# 14. Persistence

Application data must remain local.

Use an appropriate Tauri-supported local persistence mechanism.

Store:

- profiles;
- repository registrations;
- directory rules;
- user preferences.

Never store:

- GitHub tokens;
- passwords;
- private SSH keys;
- credential helper secrets.

Paths to SSH keys may eventually be stored, but never key contents.

---

# 15. Error handling

CLI operations should return structured backend errors.

Example concept:

```ts
type AppError =
  | DependencyMissing
  | GitCommandFailed
  | GithubCliCommandFailed
  | InvalidRepository
  | InvalidProfile
  | PermissionDenied
  | ConfigurationConflict
  | ParseError;
```

Do not pass raw command output directly to the UI as the main user-facing error.

Keep detailed diagnostic information available for debugging.

Never log secrets.

---

# 16. Testing

Tests are mandatory.

Frontend:

- unit tests for stores and domain logic;
- component tests where valuable;
- integration tests for critical flows.

Backend:

- Rust unit tests for:

  - command argument construction;
  - Git config parsing;
  - repository validation;
  - account parsing;
  - profile validation.

Implement abstraction boundaries so CLI execution can be mocked.

Do NOT make tests modify the developer's actual global Git configuration.

For Git integration tests, create disposable temporary repositories.

Test at minimum:

```text
repository with no local identity
repository with local identity
repository inheriting global identity
repository with incorrect identity
repository without origin
non-Git directory
multiple GitHub accounts
missing gh
missing git
malformed CLI response
CLI process failure
```

Run formatting, linting, type checking, Rust checks and tests after each milestone.

---

# 17. Security

Security is a primary requirement.

Follow least privilege.

Tauri capability configuration must expose only what the application requires.

Do not expose arbitrary filesystem access to the frontend.

Do not expose arbitrary shell execution.

Validate:

- usernames;
- hostnames;
- repository paths;
- profile IDs;
- names;
- emails.

Never construct:

```text
sh -c "<user input>"
```

Use executable + argument arrays.

Never log tokens.

Never read private SSH key contents.

Never automatically modify `.ssh/config`, `.gitconfig`, or credential stores without a clearly initiated user action.

Operations affecting global configuration must require explicit confirmation.

---

# 18. Open-source readiness

Prepare the repository to be publishable publicly.

Include:

```text
README.md
LICENSE
CONTRIBUTING.md
SECURITY.md
CODE_OF_CONDUCT.md
CHANGELOG.md
```

Use a permissive open-source license unless a better reason is documented.

README should eventually contain:

- product description;
- screenshots;
- supported platforms;
- prerequisites;
- installation;
- development setup;
- architecture;
- security model;
- contribution instructions.

Do not publish anything automatically.

Prepare GitHub workflows for:

- frontend checks;
- Rust checks;
- tests;
- build verification;
- cross-platform Tauri builds.

Later add a release workflow capable of producing:

- macOS builds;
- Windows builds;
- Linux builds.

---

# 19. Platform support

Target:

1. macOS
2. Windows
3. Linux

Development may start on macOS, but architecture and filesystem handling must not assume POSIX semantics.

Use cross-platform path APIs.

Do not hardcode:

```text
/home/...
/Users/...
C:\...
```

---

# 20. Application architecture

Keep clear boundaries.

Suggested conceptual structure:

```text
UI
↓
Application services
↓
Tauri IPC
↓
Rust commands
↓
GitService / GithubCliService / ConfigService
↓
git / gh / filesystem
```

Do not spread CLI execution logic throughout individual Tauri commands.

Prefer services that can be independently tested.

The frontend should consume typed DTOs rather than parse CLI output.

---

# 21. UX requirements

Every destructive or configuration-changing operation should distinguish:

```text
Current state
Desired state
Changes
Result
```

Example:

```text
Fix identity

Repository
~/Developer/company/project

Current
Personal
personal@example.com

New
Work
john@company.com

Changes
user.name  John Doe
user.email john@company.com

[Cancel] [Apply]
```

After applying, immediately re-read the actual Git configuration and verify success.

Never assume a command succeeded simply because the process returned no obvious error.

---

# 22. Accessibility and keyboard UX

The application should be fully usable using keyboard navigation.

Plan command-palette support later.

At minimum support:

- correct focus states;
- accessible labels;
- semantic controls;
- sensible tab order;
- keyboard activation;
- sufficient contrast.

---

# 23. Development process

Do not blindly implement the entire product in one uncontrolled pass.

Work through milestones.

For every milestone:

1. inspect the existing repository;
2. state the milestone objective;
3. identify affected files;
4. implement;
5. run tests;
6. run lint/typecheck;
7. run Rust checks;
8. run the Tauri application/build where applicable;
9. inspect failures;
10. fix failures;
11. provide a concise completion report.

Never claim a milestone is complete unless verification has actually been run.

Do not hide failing tests.

Do not weaken tests to make them pass.

Do not introduce mocks into production code just to satisfy tests.

---

# 24. Milestones

## Milestone 0 — Architecture and bootstrap

Before implementing product features:

- inspect environment;
- check installed versions of Node, pnpm, Rust, Cargo, Git and gh;
- research current official Tauri 2 and Svelte documentation;
- initialize project;
- establish architecture;
- configure TypeScript strict mode;
- configure formatting/linting/testing;
- create AGENTS.md;
- create initial architecture documentation.

Create:

```text
docs/
├── ARCHITECTURE.md
├── PRODUCT.md
├── SECURITY.md
└── MILESTONES.md
```

Do not over-design.

---

## Milestone 1 — Environment detection

Implement:

- Git detection;
- GitHub CLI detection;
- versions;
- application health status;
- typed Rust → frontend API;
- dependency status UI.

Acceptance:

The app starts and accurately reports whether `git` and `gh` exist.

---

## Milestone 2 — GitHub accounts

Implement:

- parse `gh auth status --json hosts`;
- list GitHub accounts;
- indicate active account;
- switch account;
- refresh state;
- launch login flow.

Acceptance:

A user with multiple GitHub CLI accounts can switch active account through the UI and the app verifies that the selected account became active.

---

## Milestone 3 — Profiles

Implement:

- profile persistence;
- create;
- edit;
- delete;
- GitHub account association;
- profile cards;
- validation.

Acceptance:

Profiles survive app restart and contain no authentication secrets.

---

## Milestone 4 — Repository inspection

Implement:

- native folder selection;
- repository validation;
- repository registration;
- origin inspection;
- current Git identity;
- configuration source;
- repository UI.

Acceptance:

The application can accurately describe the effective Git identity of a selected repository.

---

## Milestone 5 — Repository profile assignment

Implement:

- assign profile;
- mismatch detection;
- preview changes;
- apply local Git identity;
- verify result.

Acceptance:

A mismatched repository can be corrected through the UI without changing unrelated global configuration.

---

## Milestone 6 — Directory rules

Implement:

- path-based rules;
- conditional Git configuration where appropriate;
- safe configuration editing;
- backups;
- verification;
- conflict detection.

Acceptance:

Repositories inside a configured directory automatically resolve to the expected identity without manual switching.

---

## Milestone 7 — Polish

Implement:

- keyboard UX;
- empty states;
- first-run experience;
- error messages;
- dark/light themes;
- loading states;
- visual polish.

---

## Milestone 8 — Open-source release readiness

Implement:

- README;
- LICENSE;
- CONTRIBUTING;
- SECURITY policy;
- CI;
- build matrix;
- release workflow;
- developer documentation.

Do not publish or create a GitHub release without explicit authorization.

---

# 25. Initial task

Start with Milestone 0.

Before changing files:

1. inspect the repository and development environment;
2. read relevant official documentation for current versions;
3. create a concise implementation plan;
4. identify assumptions.

Then implement Milestone 0 completely.

At the end provide:

```text
Milestone 0 report

Architecture:
...

Files created:
...

Commands executed:
...

Tests/checks:
...

Known limitations:
...

Next milestone:
Milestone 1 — Environment detection
```

Do not start Milestone 1 yet.

Wait for the next instruction after completing and verifying Milestone 0.
