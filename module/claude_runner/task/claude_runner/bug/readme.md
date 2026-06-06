<!-- bug_system_metadata
highest_id: 250
-->

# Bug Registry — claude_runner

### Scope

Filed bug reports for the `claude_runner` crate. Each file documents a confirmed or filed defect with symptom, root cause analysis, and fix location. Bug IDs use the global BUG-NNN namespace shared across the workspace (see source code comments for historical bugs BUG-037 through BUG-246).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `247_stdout_swallowed_on_failure.md` | `run_print_mode()` discards stdout content when exit_code != 0 |
| `248_keep_claudecode_no_warning.md` | No warning when `--keep-claudecode` disables CLAUDECODE protection |
| `249_ask_help_hits_session_gate.md` | `clr ask help` blocks on session gate instead of showing help |
| `250_guard_misses_edit_distance_1_typos.md` | Guard misses one-character mid-word insertion typos (e.g. "assk" for "ask") |

### Index

| ID | Title | Status | Date | File |
|----|-------|--------|------|------|
| BUG-247 | `run_print_mode()` discards stdout content when exit_code != 0 | Verified | 2026-06-07 | [247_stdout_swallowed_on_failure.md](247_stdout_swallowed_on_failure.md) |
| BUG-248 | No warning when `--keep-claudecode` disables CLAUDECODE protection | Verified | 2026-06-07 | [248_keep_claudecode_no_warning.md](248_keep_claudecode_no_warning.md) |
| BUG-249 | `clr ask help` hits session gate instead of showing help | Verified | 2026-06-07 | [249_ask_help_hits_session_gate.md](249_ask_help_hits_session_gate.md) |
| BUG-250 | Guard misses edit-distance-1 typos ("assk" for "ask") | Verified | 2026-06-07 | [250_guard_misses_edit_distance_1_typos.md](250_guard_misses_edit_distance_1_typos.md) |
