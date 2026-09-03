# Security Policy

Git Identity Manager handles Git and GitHub identity configuration on a
developer's machine. Security is a primary requirement — see
[`docs/SECURITY.md`](docs/SECURITY.md) for the full security model and threat
boundary.

## Supported versions

The project is pre-1.0. Only the latest `0.x` release line receives security
fixes.

| Version | Supported |
| ------- | --------- |
| `0.1.x` | ✅        |
| older   | ❌        |

## Reporting a vulnerability

**Please do not open a public issue for security problems.**

Report privately through GitHub's private vulnerability reporting:

1. Open the repository's **Security** tab.
2. Choose **Report a vulnerability**.
3. Fill in the advisory form.

If private reporting is unavailable to you, open a minimal public issue that
says only "security report — please enable private reporting" with no details,
and a maintainer will follow up.

### What to include

- Affected version, OS, and build (from source vs. release binary).
- A description of the issue and its impact.
- Reproduction steps or a proof of concept.
- Any relevant logs (with tokens, emails, and paths redacted).

### What to expect

- Acknowledgement within about a week.
- An assessment and, if accepted, a fix targeted at the current `0.x` line.
- Credit in the release notes and advisory unless you ask otherwise.

## Scope

In scope: anything that breaks the documented trust boundary — for example
arbitrary command or filesystem access reachable from the frontend, exposure or
persistence of tokens / passwords / private keys / credential-helper secrets,
unprompted modification of global Git, SSH, or credential configuration, or
injection into `git` / `gh` argument construction.

Out of scope: vulnerabilities in `git`, the GitHub CLI, the operating system
credential store, or other third-party software (report those upstream), and
issues that require an already-compromised local account.
