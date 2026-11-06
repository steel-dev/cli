# Steel CLI Documentation

This folder contains both generated command docs and hand-maintained migration/reference docs.

## Docs Map

- `cli-reference.md`: auto-generated CLI command reference.
- `references/`: stable quick-reference docs.
  - Profile commands and persistence are documented in `references/steel-browser.md` and `references/steel-cli.md`.

## References Subfolder

- `references/steel-cli.md`: high-level CLI behavior and command-group reference.
- `references/steel-browser.md`: browser modes, lifecycle contracts, and how commands run.
- `references/steel-browser-commands.md`: `steel browser` command catalog.

## Regenerating Generated Docs

Run:

```bash
npm run docs:generate
```

This builds the project and regenerates `docs/cli-reference.md` from command schemas.

## Notes

- Only `cli-reference.md` is generated.
- All other docs in this folder are hand-maintained and should be updated when behavior changes.
