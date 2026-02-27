# Steel CLI Documentation

This folder contains both generated command docs and hand-maintained migration/reference docs.

## Docs Map

- `cli-reference.md`: auto-generated CLI command reference.
- `browser-compat.md`: Steel Browser compatibility contract and caveats.
- `migration-agent-browser.md`: migration guide from `agent-browser` to `steel browser`.
- `upstream-sync.md`: maintainer guide for vendored runtime updates.
- `references/`: stable quick-reference docs and synced upstream command catalogs.

## References Subfolder

- `references/steel-cli.md`: high-level CLI behavior and command-group reference.
- `references/steel-browser.md`: browser modes, lifecycle contracts, passthrough behavior.
- `references/steel-browser-commands.md`: synced/transformed upstream command catalog for `steel browser`.

## Regenerating Generated Docs

Run:

```bash
npm run docs:generate
```

This builds the project and regenerates `docs/cli-reference.md` from command schemas.

## Notes

- Only `cli-reference.md` is generated.
- All other docs in this folder are hand-maintained and should be updated when behavior changes.
