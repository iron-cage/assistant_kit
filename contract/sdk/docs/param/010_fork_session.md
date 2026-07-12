# forkSession

Creates a new session UUID when resuming, leaving the original session file unchanged.

### Forms

| | Value |
|-|-------|
| TS Field | `forkSession?: boolean` |
| Python Field | `fork_session` (inferred snake_case-identical) |
| CLI Equivalent | `--fork-session` — [`../../../claude_code/docs/behavior/021_b21_fork_session.md`](../../../claude_code/docs/behavior/021_b21_fork_session.md) |

### Type

bool

### Default

`false`

### Since

SDK GA

### Description

Field-for-field equivalent of the CLI's `--fork-session` flag (behavior [B21](../../../claude_code/docs/behavior/021_b21_fork_session.md)). Used together with [`007_resume.md`](007_resume.md) or [`009_continue.md`](009_continue.md) — the incoming session's history is loaded as context, but new entries are written to a fresh session file/UUID rather than appended to the original, enabling a "branch and explore" workflow (e.g. try two different fixes from the same starting context without either polluting the other's transcript).

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master curated parameter table |
| doc | [007_resume.md](007_resume.md) | Typically combined with this field |
| behavior | [../behavior/008_s8_session_identity_options_vs_flags.md](../behavior/008_s8_session_identity_options_vs_flags.md) | Full session-identity field group |
| doc | [../../../claude_code/docs/behavior/021_b21_fork_session.md](../../../claude_code/docs/behavior/021_b21_fork_session.md) | CLI-level equivalent behavior |
