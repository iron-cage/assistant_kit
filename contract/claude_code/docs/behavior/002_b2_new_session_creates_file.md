# Behavior B2: New Session Creates File

### Scope

- **Purpose**: Document that each `claude` invocation without `--continue` creates a distinct new `.jsonl` session file.
- **Responsibility**: Authoritative instance for behavior B2 — defines the behavior statement, certainty level, and supporting evidence.
- **In Scope**: One new `.jsonl` per independent invocation; `--new-session` as `clr` wrapper flag only (not a binary flag).
- **Out of Scope**: Binary default new-session behavior (→ [B1](001_b1_default_new_session.md)); long-term accumulation without rotation (→ [B6](006_b6_session_accumulation.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 95% | **Tier**: VALIDATED | **Since**: pre-v1.0 | **Evidence**: E1, E12, E47

Each `claude` invocation without `--continue` creates a separate new `.jsonl` session file; sessions are never appended to existing files unless explicitly continued.

`--new-session` is a **`clr` wrapper flag** — it is absent from `claude --help`. When the `clr` wrapper passes this flag, it suppresses the wrapper's default `-c` to restore the binary's native behavior (fresh session). It has no meaning to the `claude` binary directly.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E1 | B1, B2 | Code | `../../../../module/claude_runner/src/main.rs` | line 85 | `--new-session  Start a new session (default: continues previous)` — `clr` wrapper help text |
| E12 | B2 | Test | `../../tests/behavior/b02_new_session.rs` | `b2_multiple_session_files_exist_in_real_project` | At least one project in real `~/.claude/` storage has 2+ non-empty non-agent `.jsonl` files |
| E47 | B1, B2 | Test | `../../tests/behavior/b02_new_session.rs` | `b2_continue_flag_proves_separate_sessions` | `--continue` flag exists in `claude --help` — binary-level proof that new-session is the default; `--new-session` is absent from binary help (wrapper-only flag confirmed by E1) |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [001_b1_default_new_session.md](001_b1_default_new_session.md) | Binary default is new session |
| behavior | [006_b6_session_accumulation.md](006_b6_session_accumulation.md) | Long-term accumulation pattern |
| behavior | [008_b8_zero_byte_placeholder.md](008_b8_zero_byte_placeholder.md) | Zero-byte placeholder file created at session start |
| test | `../../tests/behavior/b02_new_session.rs` | Invalidation test |
