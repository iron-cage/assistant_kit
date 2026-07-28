# continue

Resume the most recently modified session without specifying its ID.

### Forms

| | Value |
|-|-------|
| TS Field | `continue?: boolean` |
| Python Field | `continue` is a Python reserved word — the actual Python field name was not confirmed in the pages fetched for this crate; treat as unconfirmed, likely `continue_conversation` or similar by analogy with `claude_runner_core`'s own Rust field naming (`continue_conversation`, per `contract/claude_code/docs/behavior/readme.md` Evidence E2) |
| CLI Equivalent | `-c` / `--continue` — [`../../../claude_code/docs/behavior/004_b4_continue_flag.md`](../../../claude_code/docs/behavior/004_b4_continue_flag.md) |

### Type

bool

### Default

`false`

### Since

SDK GA

### Description

Field-for-field equivalent of the CLI's `-c`/`--continue` flag (behavior [B4](../../../claude_code/docs/behavior/004_b4_continue_flag.md)) — resumes the mtime-most-recent session (see [B5](../../../claude_code/docs/behavior/005_b5_mtime_selection.md)) rather than a specifically-identified one (contrast [`007_resume.md`](007_resume.md)). Flagged with a naming caveat: `continue` is a reserved keyword in Python, so the TypeScript field name documented here (`continue`) cannot be assumed to carry over unchanged to `ClaudeAgentOptions` — this is the one field in this curated set where the cross-language name mapping is genuinely uncertain rather than a confident snake_case inference.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master curated parameter table |
| doc | [007_resume.md](007_resume.md) | Specific-ID alternative to this mtime-based resume |
| behavior | [../behavior/008_s8_session_identity_options_vs_flags.md](../behavior/008_s8_session_identity_options_vs_flags.md) | Full session-identity field group |
| doc | [../../../claude_code/docs/behavior/004_b4_continue_flag.md](../../../claude_code/docs/behavior/004_b4_continue_flag.md) | CLI-level equivalent behavior |
| doc | [../../../claude_code/docs/behavior/005_b5_mtime_selection.md](../../../claude_code/docs/behavior/005_b5_mtime_selection.md) | mtime selection mechanics this field triggers |
