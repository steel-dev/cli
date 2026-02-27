# Steel Browser Troubleshooting

Use this reference when automation fails or behaves inconsistently.

## First diagnostic pass

Run:

```bash
steel browser sessions
steel browser live
```

Then verify:

- same mode across commands (cloud vs local/self-hosted),
- same `--session <name>` across workflow steps,
- auth and endpoint inputs are present.

## Common failures and fixes

### Missing auth

Symptom:

- `Missing browser auth. Run steel login or set STEEL_API_KEY.`

Fix:

```bash
steel login
```

or set `STEEL_API_KEY` in runtime environment.

### Self-hosted API unreachable

Symptom:

- Errors reaching Steel session API in local/self-hosted mode.

Fix:

1. Check endpoint flags/env resolution.
2. For local runtime flow:

```bash
steel dev install
steel dev start
steel browser start --local --session local-debug
```

### Session not reused

Symptom:

- New browser appears unexpectedly between steps.

Fix:

- Ensure exact same `--session <name>` value on every command.
- Ensure commands do not mix cloud and local mode.

### No active live session

Symptom:

- `No active live session found.`

Fix:

```bash
steel browser start --session my-job
steel browser live
```

### Stale state or stuck mapping

Fix:

```bash
steel browser stop --all
steel browser start --session fresh-job
```

## Safe CDP handling

- Treat `connect_url`/`connectUrl` as display-safe only.
- Build auth-bearing URLs from `session id` plus environment key in runtime, not from copied log output.

## Recovery pattern for agents

1. Capture failing command and exact error text.
2. Identify whether issue is auth, mode, endpoint, or session continuity.
3. Apply the smallest fix that addresses root cause.
4. Re-run prior command before attempting additional workflow steps.
