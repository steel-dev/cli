# Upstream Sync Guide (agent-browser Runtime)

This guide defines how to keep the vendored browser runtime aligned with upstream `vercel-labs/agent-browser` while keeping Steel adapter logic isolated.

## Current Source of Truth

Runtime assets are vendored under:

- `vendor/agent-browser/`

Packaged release output goes to:

- `dist/vendor/agent-browser/`

Runtime metadata is pinned in:

- `vendor/agent-browser/runtime-manifest.json`

The manifest includes upstream source metadata (`repository`, `releaseTag`, `releasedAt`) and per-platform checksums.

## Update Policy

1. Keep Steel-specific logic in adapter/lifecycle modules under `source/utils/browser/`.
2. Avoid patching vendored runtime assets in place.
3. Treat upstream runtime updates as atomic vendor updates with smoke validation.

## Update Workflow

1. Fetch candidate upstream release metadata.
2. Replace `vendor/agent-browser` runtime payload with the new release assets and manifest.
3. Run packaging and validation:

```bash
npm run build
npm run browser:runtime:smoke
npm run test:unit
```

4. If cloud auth is available, run:

```bash
npm run browser:cloud:smoke
```

5. Update docs references if release tag/compatibility assumptions changed.

## Runtime Packaging Contract

Packaging script:

- `scripts/package-browser-runtime.js`

Behavior:

- validates runtime manifest shape and relative paths
- copies platform entrypoint paths + shared paths (for example `dist/` daemon assets)
- writes packaged runtime into `dist/vendor/agent-browser`

## Adapter Ownership Boundary

Steel-owned files:

- `source/utils/browser/adapter.ts`
- `source/utils/browser/lifecycle.ts`
- `source/utils/browser/routing.ts`
- `source/commands/browser/*.tsx`
- `source/commands/dev/*.tsx`

Vendored runtime files:

- `vendor/agent-browser/**`

Do not mix these concerns in a single change unless required for compatibility repair.

## Release Gate Checklist

- Unit tests pass
- Runtime smoke passes
- Browser compatibility docs still accurate
- Migration docs still accurate
- `runtime-manifest.json` source metadata reflects the updated release
