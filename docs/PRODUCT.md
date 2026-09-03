# Product

Git Identity Manager is an open-source, local-first desktop application for developers who use multiple GitHub accounts and Git identities on the same computer.

## Goals

- Discover GitHub accounts authenticated through GitHub CLI.
- Show and switch the active GitHub CLI account.
- Define reusable Git identity profiles.
- Associate Git profiles with GitHub accounts.
- Inspect effective Git identity for repositories.
- Assign a profile to a repository using local Git config.
- Detect mismatches between expected and current identities.
- Make each underlying change understandable before applying it.

## Non-goals

- Hosted backend or cloud sync.
- Storing GitHub tokens or passwords.
- Automatic installation of Git, GitHub CLI, or system software.
- SSH identity management beyond preserving a future-ready domain model.

## Primary users

Developers who switch between personal, work, client, or open-source identities and want safer defaults than remembering Git, GitHub CLI, SSH, and per-repository commands manually.
