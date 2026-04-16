---
name: changelog
description: Polish the Unreleased section of CHANGELOG.md for a tagged release. Use when a new tag has been pushed and the unreleased section needs to become a user-facing release entry. Reads git-cliff mechanical output plus the diff between the previous tag and HEAD, then rewrites terse commit subjects into user-facing prose, groups related changes, and flags breaking changes.
---

# changelog skill

Turn the mechanical `git-cliff` output for an unreleased section into a release entry that is worth reading.

This skill runs in two places:
- **Locally** during a release, invoked by the human cutting the release.
- **In CI** via `anthropics/claude-code-action`, triggered after a tag is pushed. See `.github/workflows/changelog.yml`.

The mechanical step (`git-cliff --unreleased --tag vX.Y.Z --prepend CHANGELOG.md`) runs *before* this skill is invoked. This skill only edits the section that cliff just wrote.

## Inputs you should gather first

Before editing anything, read:

1. `CHANGELOG.md` — confirm the target section exists with `## [vX.Y.Z] - YYYY-MM-DD` header
2. The tag name (passed as arg, or read from `git describe --tags --abbrev=0`)
3. `git log <previous-tag>..<new-tag> --format="%H %s%n%b%n---"` — commit subjects and bodies
4. `git diff <previous-tag>..<new-tag> --stat` — scope of changes, to sanity-check that nothing big is missing from the bullets

## What to change

**Rewrite each bullet into a user-facing sentence.**
- "Forge command" → "Fixed `steel forge` command that failed when the target path contained spaces."
- "Simplify --agent flow" → "Simplified the `--agent` onboarding flow — one fewer prompt, and the agent guide always prints on first run."
- "Upgrade rustls-webpki to resolve RUSTSEC-2026-0098/0099" → keep mostly as-is, it's already user-facing.

Each bullet should answer: *what changed that a user upgrading would notice or care about?* If a commit has no user-visible effect, move it to a **Miscellaneous** subsection or drop it.

**Verify every claim against the diff — do not paraphrase commit intent.** For each commit whose subject you expand, open the diff with `git show <sha>` and confirm the bullet describes what actually changed. If you cannot confirm a detail from the diff, leave the bullet at the original commit subject rather than invent specifics. Hallucinated behavior is the number-one failure mode.

**Group semantically, not just by commit type.** If three fixes all relate to cookies, put them under one bullet like: "Fixed several cookie decryption issues on Linux (KWallet support, correct keyring names for Opera/Vivaldi, plaintext fallback for unrecognized prefixes)." Prefer two short bullets over one compound 50-word sentence — scannability matters.

**Classify behavior changes as `Changed`, not `Bug Fixes` — even when the commit says `fix:`.** Any commit that switches an endpoint, URL, protocol, default value, flag behavior, or output format is a *behavior change* a user needs to notice. Example: "switched template downloads from raw.githubusercontent.com to registry.steel-edge.net" belongs under `### Changed` with a migration note, not under `### Bug Fixes`.

**Surface security fixes in a dedicated `### Security` section.** Any commit referencing a CVE, RUSTSEC advisory, or a dependency upgrade whose primary purpose is patching a vulnerability goes under `### Security`. Do not bury these in `### Bug Fixes`.

**Flag breaking changes in `### Breaking Changes` at the top of the entry.** Look for:
- Removed CLI flags or commands
- Renamed options (grep the diff for `--` flag changes)
- Changed default values
- Config file schema changes
- Changed external endpoints, URLs, or protocols that users' scripts may depend on
- Minimum version bumps of user-facing dependencies

Each breaking change needs a one-line migration note.

**Preserve PR/issue links.** If the mechanical output has `(#71)`, keep it. Users click these.

## What NOT to change

- Do not touch sections for previously released versions. Only edit the current version's section.
- Do not invent features. Every bullet must trace to a real commit in the range.
- Do not remove the date — cargo-dist uses `## [vX.Y.Z] - YYYY-MM-DD` as the delimiter for release-body extraction.

## Canonical subsection headers

Use these exact headers, in this order, omitting any that are empty for this release. The mechanical `git-cliff` output uses different groupings (`Features`, `Bug Fixes`, `Documentation`, `Refactor`, `Miscellaneous`) — polishing re-groups into this canonical set.

1. `### Breaking Changes` — anything in the "Flag breaking changes" list, with migration notes
2. `### Security` — CVE, RUSTSEC, or vuln-motivated dependency upgrades
3. `### Features` — user-visible new capabilities
4. `### Changed` — behavior changes, renamed flags, endpoint switches, default changes
5. `### Bug Fixes` — pure fixes with no behavior change
6. `### Documentation` — notable doc updates (skip trivial README tweaks)
7. `### Miscellaneous` — internal refactors, deps, CI — one rolled-up bullet where possible

## Output shape

```markdown
## [v0.3.9] - 2026-04-20

### Breaking Changes

- `steel foo --bar` has been renamed to `steel foo --baz`. Update scripts accordingly.

### Security

- Upgraded `rustls-webpki` to patch RUSTSEC-2026-0098 and RUSTSEC-2026-0099.

### Features

- One-line user-facing description. (#123)

### Changed

- Template downloads now come from `registry.steel-edge.net` instead of raw GitHub. Scripts that pinned to raw URLs need to update.

### Bug Fixes

- Grouped, user-facing fix description.

### Miscellaneous

- Routine dependency updates (tokio, zip). (#74, #75)
```

## How to invoke

Local:
```
claude "Run the changelog skill for tag $(git describe --tags --abbrev=0)"
```

CI: handled automatically by `.github/workflows/changelog.yml` on tag push.

## Done criteria

- The section for the new tag has at least one bullet per user-visible commit in the range.
- No bullet is shorter than the commit subject it came from — if you couldn't improve it, leave the original.
- Breaking changes are at the top with migration notes.
- `CHANGELOG.md` still parses as Markdown (no broken lists, no stray backticks).
