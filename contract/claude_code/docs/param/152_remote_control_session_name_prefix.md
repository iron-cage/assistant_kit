# remote_control_session_name_prefix

Sets the prefix used when Remote Control auto-generates a session name.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--remote-control-session-name-prefix <prefix>` |
| Env Var | — |
| Config Key | — |

### Type

string

### Default

the machine hostname

### Since

≤v2.1.92 (2026-04-04) — [`../version/017_v2_1_92.md`](../version/017_v2_1_92.md): *"Remote Control session names now use your hostname as the default prefix (e.g. `myhost-graceful-unicorn`), overridable with `--remote-control-session-name-prefix`"*. That entry changes the *default* rather than introducing the flag, so the flag itself predates v2.1.92. Help-listed since v2.1.133.

### Description

Help text:

> Prefix for auto-generated Remote Control session names (default: hostname)

**Applies only to auto-generated names.** If `--remote-control <name>` supplies an explicit name, there is nothing to prefix. The parameter is meaningful only for the bare `--remote-control` form.

**The default leaks the hostname.** Auto-generated names are visible to Remote Control clients, which means the machine's hostname is by default visible there too. Setting an explicit prefix is the documented way to avoid that.

### Verification

```bash
claude --help | grep -A2 -- '--remote-control-session-name-prefix'

for f in --remote-control-session-name-prefix --nope-xyz; do
  claude -p "$f" </dev/null 2>&1 | grep -qi 'unknown option' \
    && echo "REJECTED $f" || echo "accepted $f"
done   # → accepted, REJECTED
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [151_remote_control.md](151_remote_control.md) | The flag this one modifies |
| doc | [../version/017_v2_1_92.md](../version/017_v2_1_92.md) | Release setting hostname as the default prefix |
