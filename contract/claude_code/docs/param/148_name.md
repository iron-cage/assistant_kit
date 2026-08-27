# name

Sets a human-readable display name for the session.

### Forms

| | Value |
|-|-------|
| CLI Flag | `-n`, `--name <name>` |
| Env Var | — |
| Config Key | — |

### Type

string

### Default

— (unset; the session shows its generated identifier instead)

### Since

Unverified. No changelog entry in the 2.1.74–2.1.220 window names this flag, so no introduction version can be cited. It is accepted by v2.1.220 and listed in its `--help`.

### Description

Help text:

> Set a display name for this session (shown in the prompt box, `/resume` picker, and terminal title)

**Three surfaces, one value.** The name is presentation-only across all three: the prompt box, the `/resume` picker, and the terminal title. Nothing in the help text indicates it participates in session identity — the session ID, the storage path, and resume targeting are unaffected. Treat `--name` as a label, not a key.

**Distinct from Remote Control naming.** [`151_remote_control.md`](151_remote_control.md) accepts its own optional `[name]`, and [`152_remote_control_session_name_prefix.md`](152_remote_control_session_name_prefix.md) sets a prefix for auto-generated Remote Control names. Those name a Remote Control session; `--name` names the local session's display. Whether one falls back to the other is not established here.

### Verification

```bash
claude --help | grep -A2 -- '-n, --name'

for f in -n --name --nope-xyz; do
  claude -p "$f" </dev/null 2>&1 | grep -qi 'unknown option' \
    && echo "REJECTED $f" || echo "accepted $f"
done   # → accepted, accepted, REJECTED
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [151_remote_control.md](151_remote_control.md) | Remote Control session naming (different namespace) |
| doc | [152_remote_control_session_name_prefix.md](152_remote_control_session_name_prefix.md) | Prefix for auto-generated Remote Control names |
| doc | [../taxonomy/readme.md](../taxonomy/readme.md) | Project/Conversation/Session/Entry hierarchy |
