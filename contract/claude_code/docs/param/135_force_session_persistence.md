# force_session_persistence

Override the child-session resume/history exclusion when it is a false positive.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_FORCE_SESSION_PERSISTENCE` |
| Config Key | — |

### Type

bool

### Default

`false` (unset)

### Since

≤v2.1.197 (documented); the most common false-positive case (`tmux`) is
auto-detected and fixed without this override as of v2.1.178+

### Description

Set to `1` to force a session to register normally on `--resume`/
`--continue`/history/`claude agents` even though
[`CLAUDE_CODE_CHILD_SESSION`](133_child_session.md) is present and would
otherwise exclude it.

The concrete false-positive scenario: the Bash tool starts a `screen`
session; that subprocess inherits `CLAUDE_CODE_CHILD_SESSION=1` from the
parent tool call; a human later attaches to the `screen` session and starts a
genuinely-intentional interactive `claude` session inside it. Because the env
var is still inherited across the attach, the new session would incorrectly
be treated as machine-spawned scaffolding and excluded from resume/history
surfaces — unless this override is set, forcing normal registration.

v2.1.178+ auto-detects and corrects this specifically for `tmux`, so `tmux`
no longer needs the manual override. `screen` and other terminal
multiplexers/wrappers without equivalent auto-detection still do.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [133_child_session.md](133_child_session.md) | The exclusion this variable overrides |
| doc | [132_claudecode.md](132_claudecode.md) | Broader spawned-subprocess marker |
| doc | [067_tmux.md](067_tmux.md) | `--tmux` — the wrapper case with v2.1.178+ auto-detection |
