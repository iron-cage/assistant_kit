# remote_control

Starts an interactive session reachable from Remote Control clients.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--remote-control [name]` |
| Env Var | — |
| Config Key | — |

### Type

bool with optional string argument (the session name)

### Default

off when absent; enabled with an auto-generated name when passed bare

### Since

≤v2.1.76 (2026-03-13) — [`../version/003_v2_1_76.md`](../version/003_v2_1_76.md) already carries Remote Control *fixes*, so the feature predates the changelog window. The flag was **absent from `claude --help`** until v2.1.133 ([`../version/045_v2_1_133.md`](../version/045_v2_1_133.md): *"`claude --help` now lists `--remote-control` alongside `--remote-control-session-name-prefix`"*).

### Description

Help text:

> Start an interactive session with Remote Control enabled (optionally named)

**Naming is optional, and defaults are structured.** Passing the flag bare produces an auto-generated name of the form `<prefix>-<generated>` — v2.1.92 gives the example `myhost-graceful-unicorn`, hostname-prefixed by default. The prefix is overridable via [`152_remote_control_session_name_prefix.md`](152_remote_control_session_name_prefix.md); the optional `[name]` here replaces the whole name.

**Interactive only.** The help text says *interactive session* — this is not a `--print`-mode facility.

**Survives retire→wake.** v2.1.169: background sessions preserve `--remote-control` (alongside `--ide`, `--chrome`, `--bare`) across a retire→wake transition.

**Known operational failure modes**, from the changelog — useful because they are the ones that look like bugs in your own code:

- v2.1.76: sessions silently dying when the server reaps an idle environment; rapid messages queued one-at-a-time instead of batched.
- v2.1.110: a generic error instead of a re-login prompt when the session is too old; renames from claude.ai not persisting the title locally.

### Verification

```bash
claude --help | grep -A2 -- '--remote-control '   # → the description above

for f in --remote-control --nope-xyz; do
  claude -p "$f" </dev/null 2>&1 | grep -qi 'unknown option' \
    && echo "REJECTED $f" || echo "accepted $f"
done   # → accepted, REJECTED
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [152_remote_control_session_name_prefix.md](152_remote_control_session_name_prefix.md) | Prefix for auto-generated names |
| doc | [148_name.md](148_name.md) | `-n` / `--name` — local session display name (different namespace) |
| doc | [../version/045_v2_1_133.md](../version/045_v2_1_133.md) | Release that added the flag to `--help` |
