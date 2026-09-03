# Summary

<!-- What does this change and why? -->

## Related issues

<!-- e.g. Closes #123 -->

## How was this verified?

<!-- Commands run, manual testing, new tests added. -->

## Checklist

- [ ] `pnpm format:check`, `pnpm lint`, `pnpm check`, `pnpm test`, `pnpm build` pass
- [ ] `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` pass (in `src-tauri`)
- [ ] Tests added or updated for the change (not weakened or skipped)
- [ ] No tokens, passwords, private keys, or credential-helper secrets are stored, logged, or sent to the frontend
- [ ] No new arbitrary shell or filesystem capability is exposed over IPC
- [ ] `CHANGELOG.md` and relevant `docs/` pages updated for user-visible or architectural changes
