# Behavior B4: Continue Flag Resumes Most Recent Session

### Scope

- **Purpose**: Document that `-c`/`--continue` is the explicit opt-in for resuming the most recently modified session.
- **Responsibility**: Authoritative instance for behavior B4 — defines the behavior statement, certainty level, and supporting evidence.
- **In Scope**: `-c`/`--continue` as explicit continuation opt-in; relationship to binary default (new session).
- **Out of Scope**: Mechanism by which "most recent" is determined (→ [B5](005_b5_mtime_selection.md)); resume-by-UUID override (→ [B19](019_b19_resume_flag.md)).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 85% | **Tier**: FLAG-VFY | **Evidence**: E2, E14

`-c` / `--continue` is the explicit opt-in for resuming the most recently modified session. At the binary level, continuation is NOT the default — it must be requested with `-c`. The `clr` wrapper inverts this by passing `-c` by default (see B1).

The flag is documented in `claude --help` and is a first-class CLI parameter with its own `params/` entry.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E2 | B1, B4 | Code | `../../../../module/claude_runner_core/src/command.rs` | line 600 | `if self.continue_conversation { parts.push("-c") }` — `-c` is a builder option wrapping the native flag |
| E14 | B4 | Test | `../../tests/behavior/b04_continue_flag.rs` | `b4_continue_flag_documented_in_help` | `claude --help` documents `-c` / `--continue` flag |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [001_b1_default_new_session.md](001_b1_default_new_session.md) | Binary default is new session; `clr` inverts default by passing `-c` |
| behavior | [005_b5_mtime_selection.md](005_b5_mtime_selection.md) | How "most recent" session is selected |
| behavior | [019_b19_resume_flag.md](019_b19_resume_flag.md) | `--resume`/`-r` for selecting session by UUID |
| test | `../../tests/behavior/b04_continue_flag.rs` | Invalidation test |
