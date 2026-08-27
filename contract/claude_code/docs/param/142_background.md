# background

Starts the session as a background agent and returns immediately.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--bg`, `--background` (aliases; help lists `--bg` first) |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`false`

### Since

≤v2.1.144 (2026-05-21) — present but **absent from `claude --help`** until v2.1.187 ([`../version/088_v2_1_187.md`](../version/088_v2_1_187.md) records the fix: *"Fixed `claude --help` not listing the `--bg`/`--background` flag"*). The changelog window (2.1.74–2.1.220) contains no introduction entry, so the flag predates it.

### Description

Help text:

> Start the session as a background agent and return immediately (manage with `claude agents`)

**Gated, not unconditional.** v2.1.144 notes that rejection messages *"now name the specific gate (non-TTY, env var, or setting) instead of a generic message"* — so three distinct conditions can refuse a `--bg` launch. Which env var and which setting are the gates is not established here.

**Environment propagation.** v2.1.206 fixed `CLAUDE_CODE_EXTRA_BODY` being silently ignored by `--bg` workers; the shell-exported override now follows the dispatching session. Treat env inheritance into background workers as version-sensitive.

**Flag survival across retire→wake.** v2.1.169: background sessions preserve `--ide`, `--chrome`, `--bare`, `--remote-control`, and other flags across a retire→wake transition.

### Verification

```bash
claude --help | grep -A2 -- '--bg'            # → both aliases on one line

for f in --bg --background --nope-xyz; do
  claude -p "$f" </dev/null 2>&1 | grep -qi 'unknown option' \
    && echo "REJECTED $f" || echo "accepted $f"
done   # → accepted, accepted, REJECTED
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [136_disable_background_tasks.md](136_disable_background_tasks.md) | Env var disabling background task functionality |
| doc | [140_auto_background_tasks.md](140_auto_background_tasks.md) | Force-enable automatic backgrounding |
| doc | [../subcommand/readme.md](../subcommand/readme.md) | `claude agents` — manages the sessions this flag creates |
