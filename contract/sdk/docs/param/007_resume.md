# resume

Resume a specific prior session by ID.

### Forms

| | Value |
|-|-------|
| TS Field | `resume?: string` |
| Python Field | `resume` (confirmed via official Sessions example: `ClaudeAgentOptions(resume=session_id)`) |
| CLI Equivalent | `-r` / `--resume` — [`../../../claude_code/docs/behavior/019_b19_resume_flag.md`](../../../claude_code/docs/behavior/019_b19_resume_flag.md) |

### Type

string (session UUID)

### Default

`undefined`

### Since

SDK GA

### Description

Field-for-field equivalent of the CLI's `-r`/`--resume <session-id>` flag (behavior [B19](../../../claude_code/docs/behavior/019_b19_resume_flag.md)). The official Sessions example demonstrates the canonical two-call pattern: capture `session_id` off the first `query()`'s `SDKSystemMessage` `"init"` subtype, then pass it as `resume` on a second, later `query()` call to continue with full context — the exact "list a session, resume it later" shape `claude_runner_core` already implements against the CLI flags. See [S8](../behavior/008_s8_session_identity_options_vs_flags.md) for the full session-identity field group this belongs to.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master curated parameter table |
| doc | [008_session_id.md](008_session_id.md) | Companion field — the ID this one resumes |
| doc | [009_continue.md](009_continue.md) | Related "resume most recent" alternative |
| behavior | [../behavior/008_s8_session_identity_options_vs_flags.md](../behavior/008_s8_session_identity_options_vs_flags.md) | Full session-identity field group |
| doc | [../../../claude_code/docs/behavior/019_b19_resume_flag.md](../../../claude_code/docs/behavior/019_b19_resume_flag.md) | CLI-level equivalent behavior |
