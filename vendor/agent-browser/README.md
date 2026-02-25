# Vendored Agent Browser Runtime

This directory contains vendored upstream `agent-browser` runtime binaries.

`npm run browser:runtime:package` copies artifacts from this directory into
`dist/vendor/agent-browser` using `runtime-manifest.json`.

Current runtime source:

- Repository: `https://github.com/vercel-labs/agent-browser`
- Release tag: `v0.14.0`

Layout:

```
vendor/agent-browser/
  runtime-manifest.json
  runtimes/
    darwin-arm64/
      agent-browser
    darwin-x64/
      agent-browser
    linux-arm64/
      agent-browser
    linux-x64/
      agent-browser
    win32-x64/
      agent-browser.exe
```

`runtime-manifest.json` maps each platform target to its runtime entrypoint.
