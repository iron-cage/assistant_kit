# sessionId

Assigns a deterministic session UUID to the current `query()` call.

### Forms

| | Value |
|-|-------|
| TS Field | `sessionId?: string` |
| Python Field | `session_id` (inferred snake_case-identical) |
| CLI Equivalent | `--session-id` — [`../../../claude_code/docs/behavior/020_b20_session_id_flag.md`](../../../claude_code/docs/behavior/020_b20_session_id_flag.md) |

### Type

string (UUID)

### Default

Auto-generated

### Since

SDK GA

### Description

Field-for-field equivalent of the CLI's `--session-id <uuid>` flag (behavior [B20](../../../claude_code/docs/behavior/020_b20_session_id_flag.md)). Distinct from [`007_resume.md`](007_resume.md): `sessionId` *assigns* an ID to a new session being created, while `resume` *selects* an existing session's already-assigned ID to continue. `SDKSessionInfo.sessionId` (the return type of the SDK's session-management free functions like `listSessions()`) uses the same UUID space.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master curated parameter table |
| doc | [007_resume.md](007_resume.md) | Companion field — selecting vs. assigning an ID |
| behavior | [../behavior/008_s8_session_identity_options_vs_flags.md](../behavior/008_s8_session_identity_options_vs_flags.md) | Full session-identity field group |
| doc | [../../../claude_code/docs/behavior/020_b20_session_id_flag.md](../../../claude_code/docs/behavior/020_b20_session_id_flag.md) | CLI-level equivalent behavior |
