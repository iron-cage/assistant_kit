# claudecode

Broadest marker set in any subprocess the `claude` binary itself spawns.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDECODE` |
| Config Key | — |

### Type

bool

### Default

`false` (unset) — set to `1` only inside spawned subprocesses

### Since

pre-v1.0 (documented; predates granular version tracking in this collection)

### Description

Set to `1` in **any** subprocess Claude Code spawns — Bash/PowerShell tool
invocations, `tmux`, hooks, the statusline command, stdio MCP servers — and
also set by IDE extensions in their integrated terminals. This is the
broadest and oldest identity marker: it does not distinguish "spawned by a
tool call during a session" from "a plain IDE-integrated terminal that
happens to be under Claude Code's editor extension."

[132_claudecode.md](133_child_session.md) is the narrower, more precise
marker introduced later specifically to distinguish nested `claude`
subprocesses from these other cases — prefer it when the goal is detecting
"a `claude` process launched by Claude Code itself" rather than "any
subprocess with Claude Code's fingerprint on it."

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [133_child_session.md](133_child_session.md) | Narrower marker for nested `claude` processes specifically |
| doc | [134_entrypoint.md](134_entrypoint.md) | Adjacent classifier — which wrapper launched this session |
| doc | [../behavior/029_b29_bash_claude_env.md](../behavior/029_b29_bash_claude_env.md) | Observed Bash-tool subprocess environment behavior |
