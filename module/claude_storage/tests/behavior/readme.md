# behavior

Invalidation tests for Claude Code session behaviors. Each file covers exactly one behavior
from `docs/claude_code/001_session_behaviors.md`. Tests inspect real `~/.claude/` storage to verify Claude
Code's actual output — if Claude Code changes behavior, the tests go RED.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `mod.rs` | Module registration and shared helper functions |
| `b01_default_continues.rs` | B1 — default invocation continues most recent session |
| `b02_new_session.rs` | B2 — `--new-session` creates separate `.jsonl` |
| `b03_print_flag.rs` | B3 — `-p` is output mode, not session flag |
| `b04_continue_flag.rs` | B4 — `-c` aliases default continuation |
| `b05_mtime_selection.rs` | B5 — current session selected by mtime |
| `b06_session_accumulation.rs` | B6 — sessions accumulate as separate files |
| `b07_agent_sessions.rs` | B7 — agent sessions are `agent-*.jsonl` siblings (flat layout) |
| `b08_zero_byte_init.rs` | B8 — zero-byte `.jsonl` created as placeholder on startup |
| `b09_storage_path.rs` | B9 — project path uses `/` to `-` encoding |
| `b10_entry_threading.rs` | B10 — entries linked via `parentUuid` |
| `b11_auto_continue.rs` | B11 — `CLAUDE_CODE_AUTO_CONTINUE` env var |
| `b12_agent_session_id_is_parent.rs` | B12 — agent `sessionId` equals parent UUID |
| `b13_subagent_directory_structure.rs` | B13 — `{uuid}/subagents/` hierarchy |
| `b14_agent_meta_json.rs` | B14 — `.meta.json` sidecars with `agentType` |
| `b15_agent_slug_field.rs` | B15 — agents carry shared `slug` field |
| `b16_tools_disable.rs` | B16 — `--tools ""` disables tool invocation (H1 vs H2 ❓ unresolved) |

## Running

```bash
cargo nextest run --test behavior
```
