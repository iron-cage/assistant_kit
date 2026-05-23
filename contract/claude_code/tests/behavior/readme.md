# Behavior Hypothesis Invalidation Tests

One file per behavior hypothesis from `docs/behavior/001_session_behaviors.md` (B1–B24 + B16h).
Each test inspects real `~/.claude/` storage or invokes `claude --help` / `--version` directly.
If Claude Code changes behavior, the corresponding test goes RED.

## Responsibility Table

| File | Behavior | Responsibility |
|------|----------|----------------|
| `mod.rs` | — | Test binary entry point; shared helpers for `~/.claude/` inspection |
| `b01_default_continues.rs` | B1 | Resumable (non-empty, non-agent) session exists in real storage |
| `b02_new_session.rs` | B2 | Multiple non-agent session files in real `~/.claude/` (binary creates separate `.jsonl` per invocation) |
| `b03_print_flag.rs` | B3 | `-p`/`--print` flag documented in `claude --help` |
| `b04_continue_flag.rs` | B4 | `-c`/`--continue` flag documented in `claude --help` |
| `b05_mtime_selection.rs` | B5 | Multiple sessions have distinct observable mtimes |
| `b06_session_accumulation.rs` | B6 | Sessions accumulate as separate `.jsonl` files |
| `b07_agent_sessions.rs` | B7 | Agent sessions are `agent-*.jsonl` siblings with `isSidechain:true` |
| `b08_zero_byte_init.rs` | B8 | 0-byte `.jsonl` placeholder files exist in real storage |
| `b09_storage_path.rs` | B9 | Project dir names follow `/`→`-` encoding convention |
| `b10_entry_threading.rs` | B10 | Conversation entries linked via `parentUuid` (null root, non-null chain) |
| `b11_auto_continue.rs` | B11 | `CLAUDE_CODE_AUTO_CONTINUE` env var recognized by `claude` |
| `b12_agent_session_id_is_parent.rs` | B12 | Agent `sessionId` matches parent root session UUID |
| `b13_subagent_directory_structure.rs` | B13 | Subagents stored in `{root}/{session}/subagents/` |
| `b14_agent_meta_json.rs` | B14 | `agent-*.meta.json` sidecar holds `agentType` field |
| `b15_agent_slug_field.rs` | B15 | Agent JSONL entries carry shared `slug` field |
| `b16_tools_disable.rs` | B16 | `--tools ""` flag accepted; disables tool invocation (H1 vs H2 open ❓) |
| `b17_parentuuid_self_contained.rs` | B17 | `parentUuid` orphaned-link rate < 1%; compaction-boundary exception documented |
| `b18_no_cross_session_links.rs` | B18 | First entry of every session has `parentUuid: null` |
| `b19_resume_flag.rs` | B19 | `--resume`/`-r` flag documented in `claude --help` |
| `b20_session_id_flag.rs` | B20 | `--session-id` flag documented in `claude --help` |
| `b21_fork_session_flag.rs` | B21 | `--fork-session` flag documented in `claude --help` |
| `b22_no_session_persistence_flag.rs` | B22 | `--no-session-persistence` flag documented in `claude --help` |
| `b23_session_dir_override.rs` | B23 | `CLAUDE_CODE_SESSION_DIR` env var not explicitly rejected by binary |
| `b24_from_pr_flag.rs` | B24 | `--from-pr` flag documented in `claude --help` |
| `b16h_tools_system_prompt.rs` | B16h | Live API token comparison (`lim_it` test; excluded from default filter) |
