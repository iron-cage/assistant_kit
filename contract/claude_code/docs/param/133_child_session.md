# child_session

The precise marker for a nested `claude` process Claude Code itself launched.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_CHILD_SESSION` |
| Config Key | — |

### Type

bool

### Default

`false` (unset)

### Since

v2.1.172+

### Description

Set to `1` specifically in subprocesses launched from the Bash/PowerShell/
Monitor tools, hooks, and the statusline command — **not** set for stdio MCP
servers (which are long-lived and outlive the parent session) and not set by
IDE extensions (which use the broader [`CLAUDECODE`](132_claudecode.md)
marker instead).

A nested interactive TUI started this way is auto-excluded from
`--resume`/`--continue`, session history, and the `claude agents` list — the
rationale being that a `claude` process spawned as a tool-call side effect is
presumed to be machine-driven scaffolding, not a session a human would want
to resume directly. A non-interactive `claude -p` invocation started the same
way still persists its session normally; the exclusion applies to the
resume/history surfaces specifically, not to session-file creation.

The false-positive case this creates — and the reason
[`CLAUDE_CODE_FORCE_SESSION_PERSISTENCE`](135_force_session_persistence.md)
exists — is a wrapper like `screen` or `tmux`: the Bash tool starts the
wrapper, the wrapper subprocess inherits `CLAUDE_CODE_CHILD_SESSION=1`, and a
human later attaches and starts a genuinely-intentional interactive session
inside it. Since the env var is still inherited across the attach, that new
session would incorrectly be excluded unless the override forces normal
registration. v2.1.178+ auto-detects and corrects this specifically for
`tmux`; other wrappers (`screen` included) still need the manual override.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [132_claudecode.md](132_claudecode.md) | Broader marker set in all spawned subprocesses |
| doc | [135_force_session_persistence.md](135_force_session_persistence.md) | Override for false-positive exclusion |
| doc | [017_continue.md](017_continue.md) | `--continue` — one of the surfaces this exclusion affects |
| doc | [055_resume.md](055_resume.md) | `--resume` — one of the surfaces this exclusion affects |
