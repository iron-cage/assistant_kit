# Behavior

### Scope

- **Purpose**: Catalog observed and confirmed external behaviors of the `claude` binary spanning session lifecycle, storage, runtime process model, subagent context, and tool availability.
- **Responsibility**: Master file for the `behavior` collection — lists all 36 behavior instances (B1–B35 + B16h), provides the shared evidence table (E1–E61), and links to invalidation test files.
- **In Scope**: Session continuation, flag semantics, agent layouts, entry threading, storage path encoding, cross-session relationship absence (conversation chain foundations); runtime process model (agent subagent identity, bash subprocess identity, env propagation); subagent context inheritance (CLAUDE.md injection, conversation absence, scope propagation); subagent tool availability per type (tool set differences, parent-exclusive tools); context loading (CLAUDE.md @-reference path filter, content pipeline transformations, silent failure and truncation modes).
- **Out of Scope**: Entry-level JSONL schema (→ [`../jsonl/`](../jsonl/readme.md)); storage directory architecture (→ [`../storage/`](../storage/readme.md)); filesystem paths (→ [`../filesystem/`](../filesystem/readme.md)); settings format (→ [`../settings/`](../settings/readme.md)); ancillary file formats (→ [`../format/`](../format/readme.md)); concept taxonomy (→ [`../taxonomy/`](../taxonomy/readme.md)).

### Overview Table

Adapted from hypothesis table format. Status reflects certainty of the observation, not investigation state. Certainty is based on source evidence — code inspection, observed output, or direct inference. All behaviors describe the external `claude` binary.

**Status legend:**
- ✅ Confirmed — source code or reproducible test confirms
- 🎯 Observed — seen in practice, mechanism inferred
- ❓ Uncertain — reasonable inference, unconfirmed

**Test Tier legend:**
- `VALIDATED` — test asserts on real `~/.claude/` storage structure (hard `assert!` on fields/counts)
- `FLAG-VFY` — test verifies flag exists in `--help` or is accepted without parse error
- `NEG-ONLY` — test asserts env var is NOT explicitly rejected (cannot confirm acceptance vs silent ignore)
- `UNVERIFIED` — test has no `assert!`; logs observation only; never goes RED
- `MEASURE` — live API measurement; no pass/fail; runs by default in container where `~/.claude` is mounted (`lim_it_` prefix)
- `VALIDATED†` — test proves feasibility of mechanism but not that the binary uses it

| ID | Behavior | Category | Status | Certainty | Tier | Since | Evidence |
|----|----------|----------|--------|-----------|------|-------|----------|
| [B1](001_b1_default_new_session.md) | `claude` binary defaults to NEW session; resuming requires explicit `--continue`/`-c`. `clr` wrapper inverts this default | Continuation | ✅ | 90% | VALIDATED | pre-v1.0 | E1, E2, E11, E47 |
| [B2](002_b2_new_session_creates_file.md) | Each invocation without `--continue` creates a new `.jsonl`; `--new-session` is a `clr` wrapper flag | Storage | ✅ | 95% | VALIDATED | pre-v1.0 | E1, E12, E47 |
| [B3](003_b3_print_orthogonal.md) | `-p`/`--print` controls output mode only; does not affect session selection | Flags | ✅ | 95% | FLAG-VFY | pre-v1.0 | E3, E13 |
| [B4](004_b4_continue_flag.md) | `-c`/`--continue` is explicit opt-in for resuming most recently modified session | Flags | 🎯 | 85% | FLAG-VFY | pre-v1.0 | E2, E14 |
| [B5](005_b5_mtime_selection.md) | "Current" session resumed by `--continue` is the most recently modified `.jsonl` (mtime) | Selection | 🎯 | 60% | VALIDATED† | pre-v1.0 | E4, E15 |
| [B6](006_b6_session_accumulation.md) | Sessions accumulate one file per independent invocation; never compacted or rotated | Storage | ✅ | 90% | VALIDATED | pre-v1.0 | E5, E16 |
| [B7](007_b7_agent_sessions_sibling.md) | Agent sessions are `agent-*.jsonl` siblings with `isSidechain: true` (flat layout) | Storage | ✅ | 95% | VALIDATED | pre-v1.0 | E6, E17 |
| [B8](008_b8_zero_byte_placeholder.md) | Claude Code creates zero-byte `.jsonl` placeholders on startup; remain if process crashes | Storage | 🎯 | 85% | UNVERIFIED | pre-v1.0 | E7, E18 |
| [B9](009_b9_storage_path_encoding.md) | Project sessions stored at `~/.claude/projects/{path-encoded}/`; `/` → `-` | Storage | ✅ | 95% | VALIDATED | pre-v1.0 | E8, E19 |
| [B10](010_b10_entry_threading.md) | Entries linked by `parentUuid`; root entry has `parentUuid: null` | Entries | ✅ | 95% | VALIDATED | pre-v1.0 | E9, E20 |
| [B11](011_b11_auto_continue_env.md) | `CLAUDE_CODE_AUTO_CONTINUE` env var enables automated continuation mode | Flags | 🎯 | 85% | NEG-ONLY | pre-v1.0 | E10, E21 |
| [B12](012_b12_agent_session_id.md) | Agent JSONL entries carry `sessionId` equal to the parent session UUID | Families | ✅ | 95% | VALIDATED | pre-v1.0 | E22, E26 |
| [B13](013_b13_subagent_directory.md) | New-format agents stored at `{parent-uuid}/subagents/agent-{agentId}.jsonl` | Families | ✅ | 95% | VALIDATED | pre-v1.0 | E23, E27 |
| [B14](014_b14_agent_meta_json.md) | Agent `.meta.json` sidecars contain `agentType` and optional `description` | Families | ✅ | 90% | VALIDATED | pre-v1.0 | E24, E28 |
| [B15](015_b15_agent_slug.md) | Agent entries carry a `slug` field shared by all agents of one parent | Families | 🎯 | 85% | VALIDATED | pre-v1.0 | E25, E29 |
| [B16](016_b16_tools_flag.md) | `--tools ""` disables all tool invocation; `--tools "default"` restores all tools | Flags | ✅ | 90% | FLAG-VFY | pre-v1.0 | E30, E31 |
| [B16h](016h_b16h_tools_system_prompt.md) | Tool definitions (~12k tokens) remain in assembled system prompt even with `--tools ""` | Flags | ❓ | 60% | MEASURE | pre-v1.0 | E32 |
| [B17](017_b17_parentuuid_self_contained.md) | `parentUuid` chain is self-contained within one session file (< 0.2% compaction exceptions) | Entries | 🎯 | 85% | VALIDATED | pre-v1.0 | E33 |
| [B18](018_b18_no_cross_session_links.md) | No cross-session continuation metadata; first entry of new session has `parentUuid: null` | Continuation | 🎯 | 80% | VALIDATED | pre-v1.0 | E34 |
| [B19](019_b19_resume_flag.md) | `--resume`/`-r` resumes a specific prior session by UUID | Continuation | 🎯 | 85% | FLAG-VFY | pre-v1.0 | E35, E36 |
| [B20](020_b20_session_id_flag.md) | `--session-id <uuid>` assigns a deterministic UUID to the current session | Session | 🎯 | 80% | FLAG-VFY | pre-v1.0 | E37, E38 |
| [B21](021_b21_fork_session.md) | `--fork-session` creates a new session UUID when resuming; original unchanged | Continuation | 🎯 | 80% | FLAG-VFY | pre-v1.0 | E39, E40 |
| [B22](022_b22_no_session_persistence.md) | `--no-session-persistence` disables session disk writes; only works with `--print` mode | Storage | 🎯 | 85% | FLAG-VFY | pre-v1.0 | E41, E42 |
| [B23](023_b23_session_dir_override.md) | `CLAUDE_CODE_SESSION_DIR` env var overrides session storage directory | Storage | 🎯 | 80% | NEG-ONLY | pre-v1.0 | E43, E44 |
| [B24](024_b24_from_pr.md) | `--from-pr [value]` resumes a session previously linked to a GitHub pull request | Continuation | 🎯 | 75% | FLAG-VFY | pre-v1.0 | E45, E46 |
| [B25](025_b25_auto_compact_window.md) | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` env var sets the effective token window for auto-compaction calculations | Flags | 🎯 | 85% | NEG-ONLY | v2.1.75 | E48, E49 |
| [B26](026_b26_autocompact_pct_override.md) | `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` env var overrides the compaction trigger as a percentage of the window | Flags | 🎯 | 80% | NEG-ONLY | v2.1.75 | E50, E51 |
| [B27](027_b27_agent_no_os_process.md) | Agent tool subagents are not OS processes; run as API inference threads within the existing claude process — `pgrep` count unchanged before/during/after | Process Model | ✅ | 99% | UNVERIFIED | v2.1.74 | E52 |
| [B28](028_b28_bash_rtk_subprocess.md) | Each Bash tool call spawns a transient `rtk` proxy process (~5 MB, 4 FDs) that exits immediately; parent PID is gone before the next call | Process Model | ✅ | 95% | UNVERIFIED | v2.1.74 | E53 |
| [B29](029_b29_bash_claude_env.md) | All bash subprocesses inherit the full parent OS environment (107 vars, identical between parent and subagent) — including all `CLAUDE_*` vars, project vars, API keys, and desktop session vars | Process Model | ✅ | 99% | UNVERIFIED | v2.1.74 | E54, E56 |
| [B30](030_b30_subagent_context_inheritance.md) | Agent subagents receive full CLAUDE.md configuration (via system-reminder injection) but not the parent conversation history; scope variables (`SCOPE_DIR`/`SCOPE_READY`) are not inherited | Context | ✅ | 99% | UNVERIFIED | v2.1.74 | E55 |
| [B31](031_b31_subagent_tool_sets.md) | Subagent tool sets are narrower than the parent session (27 tools); general-purpose lacks Agent and 13 other session-management tools; Explore and Plan are identical; claude-code-guide uses static pre-loaded tools only | Tools | ✅ | 99% | UNVERIFIED | v2.1.74 | E57 |
| [B32](032_b32_claudemd_at_ref_path_filter.md) | The `iy4()` path filter silently rejects `$VAR`/`%VAR%` and other non-conforming @-reference prefixes; only `./`, `~/`, `/`, and `[a-zA-Z0-9._-]`-initial paths are accepted; `C9()` correctly expands `~/` to `os.homedir()` | Context Loading | ✅ | 99% | UNVERIFIED | v2.1.74 | E58 |
| [B33](033_b33_claudemd_loading_limits.md) | CLAUDE.md loading fails silently for ENOENT/EISDIR/EACCES and non-whitelisted extensions; `Xm=40,000` chars is a UI-warning-only threshold (file fully injected — ~10k tokens — but interactive status bar warns); hard limits: 200-line MEMORY.md cap (`$P`), 5-level @-include depth (`ny4`), 3,000-char ultra-memory (`QKT`) | Context Loading | ✅ | 99% | UNVERIFIED | v2.1.74 | E59 |
| [B34](034_b34_claudemd_content_pipeline.md) | HTML comments stripped (`Kp6`), YAML frontmatter processed as conditional globs not injected as content, GFM disabled in @-ref lexer; `tengu_paper_halyard` Statsig flag silently drops all Project/Local CLAUDE.md; User type always bypasses external-include dialog | Context Loading | ✅ | 99% | UNVERIFIED | v2.1.74 | E60 |
| [B35](035_b35_automemory_search_context_flag.md) | `tengu_coral_fern` Statsig flag (default false) gates a `## Searching past context` section in the auto-memory system prompt — provides grep commands for memory topic files and session JSONL transcripts; absent when flag is false | Auto-Memory | ✅ | 99% | UNVERIFIED | v2.1.74 | E61 |

---

### Evidence Table

Evidence items are shared across behaviors (M:N relationship). Each item may support multiple behaviors.

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E1 | B1, B2 | Code | `../../../../module/claude_runner/src/main.rs` | line 85 | `--new-session  Start a new session (default: continues previous)` — `clr` wrapper help text; confirms wrapper default is continuation (not the `claude` binary native default) |
| E2 | B1, B4 | Code | `../../../../module/claude_runner_core/src/command.rs` | line 600 | `if self.continue_conversation { parts.push("-c") }` — `-c` is a builder option wrapping the native flag |
| E3 | B3 | Code | `../../../../module/claude_runner/src/main.rs` | lines 83, 124 | `-p, --print  Non-interactive mode` and `-p` branch sets print-only; no session flag change |
| E4 | B5 | Inference | Storage observation | `~/.claude/projects/*/` | Multiple `.jsonl` files in one project; `--continue` must pick one; mtime is the only per-file ordering signal available without metadata |
| E5 | B6 | Observation | Live storage | `~/.claude/projects/…/-commit/` | 25 `.jsonl` files observed in one project directory from repeated sessions |
| E6 | B7 | Observation | Live storage | `~/.claude/projects/*/agent-*.jsonl` | Agent session files observed as siblings of main sessions; entries contain `"isSidechain":true` |
| E7 | B8 | Observation | Live storage | `~/.claude/projects/*/` | Zero-byte `.jsonl` files observed in project directories alongside non-empty sessions |
| E8 | B9 | Observation | Live storage | `~/.claude/projects/` | Project directory names match `/`→`-` encoding of working directory paths |
| E9 | B10 | Doc | `../jsonl/009_threading_model.md` | Threading model | `parentUuid` links each entry to its parent; null on first entry of a thread |
| E10 | B11 | Code | `../../../../module/claude_runner_core/src/command.rs` | line 647-648 | `cmd.env("CLAUDE_CODE_AUTO_CONTINUE", auto_continue.to_string())` — env var set before spawning `claude` |
| E11 | B1 | Test | `../../tests/behavior/b01_default_continues.rs` | `b1_resumable_session_exists_in_real_storage` | At least one non-empty non-agent session exists in real `~/.claude/` storage — prerequisite for default continuation |
| E12 | B2 | Test | `../../tests/behavior/b02_new_session.rs` | `b2_multiple_session_files_exist_in_real_project` | At least one project in real `~/.claude/` storage has 2+ non-empty non-agent `.jsonl` files — evidence of per-session file creation |
| E13 | B3 | Test | `../../tests/behavior/b03_print_flag.rs` | `b3_print_flag_documented_as_output_mode` | `claude --help` documents `-p` / `--print` as output mode |
| E14 | B4 | Test | `../../tests/behavior/b04_continue_flag.rs` | `b4_continue_flag_documented_in_help` | `claude --help` documents `-c` / `--continue` flag |
| E15 | B5 | Test | `../../tests/behavior/b05_mtime_selection.rs` | `b5_real_sessions_have_distinct_mtimes` | Real project with 2+ sessions has distinct mtimes — mtime ordering is possible |
| E16 | B6 | Test | `../../tests/behavior/b06_session_accumulation.rs` | `b6_sessions_accumulate_in_real_project` | Real project directory contains 5+ `.jsonl` files — higher threshold than B2 to confirm long-term accumulation without rotation |
| E17 | B7 | Test | `../../tests/behavior/b07_agent_sessions.rs` | `b7_real_agent_session_has_issidechain_true` | Real `agent-*.jsonl` file contains `"isSidechain":true` in first entry |
| E18 | B8 | Observation | `../../tests/behavior/b08_zero_byte_init.rs` | `b8_zero_byte_jsonl_exists_in_real_storage` | Zero-byte `.jsonl` files observed in real `~/.claude/` storage (test logs observation, does not assert) |
| E19 | B9 | Test | `../../tests/behavior/b09_storage_path.rs` | `b9_project_dir_names_follow_encoding_convention` | Real project directory names start with `-` (encoded leading `/`) and decode to existing paths |
| E20 | B10 | Test | `../../tests/behavior/b10_entry_threading.rs` | `b10_first_entry_has_null_parent_uuid`, `b10_subsequent_entries_have_non_null_parent_uuid` | First conversation entry has `parentUuid:null`; second has non-null `parentUuid` referencing first |
| E21 | B11 | Test | `../../tests/behavior/b11_auto_continue.rs` | `b11_auto_continue_env_var_recognized` | Binary does not print `CLAUDE_CODE_AUTO_CONTINUE` in stderr when env var is set — negative assertion |
| E22 | B12 | Observation | Live storage | `~/.claude/projects/*/subagents/agent-*.jsonl` | Agent entry `sessionId` field equals the parent directory UUID, not the agent filename ID |
| E23 | B13 | Observation | Live storage | `~/.claude/projects/*/` | `{uuid}/subagents/agent-*.jsonl` directories observed; parent UUID in directory name matches root `{uuid}.jsonl` |
| E24 | B14 | Observation | Live storage | `~/.claude/projects/*/subagents/*.meta.json` | `meta.json` files contain `{"agentType":"Explore"}` or `{"agentType":"general-purpose"}` or `{"agentType":"Plan"}`; some include `description` |
| E25 | B15 | Observation | Live storage | `~/.claude/projects/*/subagents/agent-*.jsonl` | All sibling agent entries share identical `slug` value (e.g., `"jaunty-painting-hinton"`); root session first entry has no `slug` |
| E26 | B12 | Test | `../../tests/behavior/b12_agent_session_id_is_parent.rs` | `b12_agent_session_id_matches_parent_dir` | Agent entry `sessionId` equals the UUID from the parent directory path |
| E27 | B13 | Test | `../../tests/behavior/b13_subagent_directory_structure.rs` | `b13_subagent_dir_exists_for_root_session` | At least one root session has a matching `{uuid}/subagents/` directory |
| E28 | B14 | Test | `../../tests/behavior/b14_agent_meta_json.rs` | `b14_meta_json_contains_agent_type` | Real `.meta.json` file contains `agentType` field with known value |
| E29 | B15 | Test | `../../tests/behavior/b15_agent_slug_field.rs` | `b15_sibling_agents_share_slug` | All sibling agents under one parent share the same `slug` value |
| E30 | B16 | Observation | `claude --help` live output | `--tools` flag entry | Help text: "Specify the list of available tools from the built-in set. Use `""` to disable all tools, `"default"` to use all tools, or specify tool names" |
| E31 | B16 | Test | `../../tests/behavior/b16_tools_disable.rs` | `b16a_tools_flag_documented_in_help`, `b16b_tools_empty_string_accepted`, `b16c_tools_default_value_accepted` | Flag documented in help and accepted at CLI parse time without parse error |
| E32 | B16h | Inference | Research: Piebald-AI/claude-code-system-prompts; ClaudeLog (2026-04) | Tool assembly layer analysis | Tool definitions injected into assembled system prompt before behavioral flags are applied. `--tools` likely operates at invocation-policy layer, not definition-assembly layer. Unconfirmed: requires live token-count comparison. |
| E33 | B17 | Test | `../../tests/behavior/b17_parentuuid_self_contained.rs` | `it_parentuuid_never_crosses_session_boundary` | Rate-based check: orphaned `parentUuid` references stay below 1% across 10 projects × 5 sessions |
| E34 | B18 | Test | `../../tests/behavior/b18_no_cross_session_links.rs` | `it_first_entry_parentuuid_is_null` | First conversation entry in each session has `parentUuid: null` or absent — no cross-session continuation pointer written |
| E35 | B19 | Observation | `claude --help` live output | `--resume` flag entry | Help text documents `--resume` / `-r <session-id>` flag for resuming a specific prior session by UUID |
| E36 | B19 | Test | `../../tests/behavior/b19_resume_flag.rs` | `b19_resume_flag_documented_in_help` | `claude --help` output contains `--resume` flag |
| E37 | B20 | Observation | `claude --help` live output | `--session-id` flag entry | Help text documents `--session-id <uuid>` flag for assigning a deterministic UUID to the current session |
| E38 | B20 | Test | `../../tests/behavior/b20_session_id_flag.rs` | `b20_session_id_flag_documented_in_help` | `claude --help` output contains `--session-id` flag |
| E39 | B21 | Observation | `claude --help` live output | `--fork-session` flag entry | Help text documents `--fork-session` flag for branching from a prior session without modifying the original |
| E40 | B21 | Test | `../../tests/behavior/b21_fork_session_flag.rs` | `b21_fork_session_flag_documented_in_help` | `claude --help` output contains `--fork-session` flag |
| E41 | B22 | Observation | `claude --help` live output | `--no-session-persistence` flag entry | Help text documents `--no-session-persistence` flag; notes it disables `.jsonl` creation and works only with `--print` mode |
| E42 | B22 | Test | `../../tests/behavior/b22_no_session_persistence_flag.rs` | `b22_no_session_persistence_flag_documented_in_help` | `claude --help` output contains `--no-session-persistence` flag |
| E43 | B23 | Doc | `../param/057_session_dir.md` | Description | Documents `CLAUDE_CODE_SESSION_DIR` env var that overrides session storage directory |
| E44 | B23 | Test | `../../tests/behavior/b23_session_dir_override.rs` | `b23_session_dir_env_var_not_rejected` | Binary does not explicitly reject `CLAUDE_CODE_SESSION_DIR` env var at startup |
| E45 | B24 | Observation | `claude --help` live output | `--from-pr` flag entry | Help text documents `--from-pr [value]` flag for resuming sessions linked to GitHub pull requests |
| E46 | B24 | Test | `../../tests/behavior/b24_from_pr_flag.rs` | `b24_from_pr_flag_documented_in_help` | `claude --help` output contains `--from-pr` flag |
| E47 | B1, B2 | Test | `../../tests/behavior/b02_new_session.rs` | `b2_continue_flag_proves_separate_sessions` | `--continue` flag exists in `claude --help` — binary-level proof that new-session is the default; presence of a dedicated resume flag implies sessions are separate by default |
| E48 | B25 | Doc | Official Claude Code documentation (code.claude.com/docs/en/env-vars) | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` entry | "Set the context capacity in tokens used for auto-compaction calculations. Defaults to the model's context window: 200K for standard models or 1M for extended context models." |
| E49 | B25 | Test | `../../tests/behavior/b25_auto_compact_window.rs` | `b25_auto_compact_window_env_var_recognized` | Binary exits 0 and does not emit rejection referencing `CLAUDE_CODE_AUTO_COMPACT_WINDOW` when env var is set — negative assertion |
| E50 | B26 | Doc | Official Claude Code documentation (code.claude.com/docs/en/env-vars) | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` entry | "`CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` is applied as a percentage of this value" |
| E51 | B26 | Test | `../../tests/behavior/b26_autocompact_pct_override.rs` | `b26_autocompact_pct_override_env_var_recognized` | Binary exits 0 and does not emit rejection referencing `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` when env var is set — negative assertion |
| E52 | B27 | Experiment | Live `pgrep` snapshot — this session (2026-06-28) | Parent session, pre/during/post agent dispatch | `pgrep -a claude` returned 13 processes before launching 2 background agents; 13 during active execution (agents running Bash tool calls); 13 after completion. Net delta: 0. Agent Bash call PIDs (3348183, 3356028, 3373973) absent from `pgrep -a claude` output. |
| E53 | B28 | Experiment | `/proc/self/status` inspection — this session (2026-06-28) | Agent A and B Bash tool calls | `Name: rtk`, `Pid: 3349457`, `VmRSS: 4884 kB`, `Threads: 1`; `ls /proc/self/fd \| wc -l` = 4; parent PID gone before next command; `cat /proc/self/cmdline` rewrote to `rtk read /proc/self/cmdline`; `$$` empty in some invocations due to rtk interception |
| E54 | B29 | Experiment | `/proc/self/environ` inspection — this session (2026-06-28) | Agent A Bash tool call | `cat /proc/self/environ \| tr '\0' '\n' \| grep -i claude` returned 9 vars: CLAUDECODE=1, CLAUDE_CODE_ENTRYPOINT=cli, CLAUDE_TOOL_TIMEOUT=7200000, CLAUDE_EXEC_TIMEOUT=7200000, CLAUDE_BASH_TIMEOUT=7200000, CLAUDE_DEFAULT_TIMEOUT=7200000, CLAUDE_CODE_EFFORT_LEVEL=max, CLAUDE_COMMAND_TIMEOUT=7200000, CLAUDE_CODE_MAX_OUTPUT_TOKENS=100000 |
| E55 | B30 | Experiment | Dual MAAV agent experiment — this session (2026-06-28) | Agents ae4bc9897199f0fef (probe) and a4ee9bfe2aedf5c12 (adversarial) | Probe agent answered 10/10 CLAUDE.md knowledge questions YES before reading any files (2-space indent, cargo fmt forbidden, scope command, MAAV, kbase — all known from system-reminder injection). Re-read `~/.claude/CLAUDE.md` and confirmed content matched context exactly. Adversarial agent confirmed zero knowledge of parent conversation; JSONL starts at `parentUuid: null`, `isSidechain: true`. `SCOPE_DIR`/`SCOPE_READY`/`SCOPE_LEVEL` absent from both agents' environments. |
| E56 | B29 | Experiment | Full env comparison — parent vs general-purpose subagent (2026-06-29) | `cat /proc/self/environ \| tr '\0' '\n' \| sort` in both parent and subagent Bash calls | 107 variables enumerated in each context; zero differences between parent and subagent. Full environment inherited: project vars ($PRO, $GENAI, FIRECRAWL_API_KEY, etc.), non-CLAUDE_* timeouts (COMMAND_TIMEOUT=7200, TOOL_TIMEOUT=7200), NVM, desktop session (XDG_*, GNOME_*, DISPLAY), GIT_EDITOR, SSH_AUTH_SOCK, all system vars. |
| E57 | B31 | Experiment | 4-agent parallel tool inventory — this session (2026-06-29) | Agents a0421c818fd857c2b (general-purpose), a5c1902758f7bef17 (Explore), afa16d2f3f479ce74 (Plan), a4e092d7ff1371904 (claude-code-guide) | Each agent reported its complete available-deferred-tools list verbatim. general-purpose: 12 deferred + ToolSearch pre-loaded (no Agent tool). Explore: 9 deferred + ToolSearch. Plan: 9 deferred + ToolSearch (identical to Explore). claude-code-guide: 5 pre-loaded only, no ToolSearch, no deferred. Parent session: 26 deferred + ToolSearch = 27. |
| E58 | B32 | Code | Binary analysis — `strings /home/user1/.local/share/claude/versions/2.1.74` — v2.1.74 session (2026-06-29) | `iy4()` at strings line 492301; `C9()` at binary offset 108,423,272 | `iy4()` path filter: `j.startsWith("./") \|\| j.startsWith("~/") \|\| (j.startsWith("/") && j !== "/") \|\| (!j.startsWith("@") && !j.match(/^[#%^&*()]+/) && j.match(/^[a-zA-Z0-9._-]/))`. `C9()` resolver: `if(K==="~")return Uo_.homedir().normalize("NFC"); if(K.startsWith("~/"))return SZ.join(Uo_.homedir(),K.slice(2)).normalize("NFC"); if(SZ.isAbsolute(O))return SZ.normalize(O).normalize("NFC"); return SZ.resolve(q,O).normalize("NFC")`. Regex: `/(?:^|\s)@((?:[^\s\\]|\\ )+)/g`. Code/codespan skip: `if(H.type==="code"\|\|H.type==="codespan")continue`. Fragment strip: `let w=j.indexOf("#"); if(w!==-1)j=j.substring(0,w)`. |
| E59 | B33 | Code | Binary analysis — `strings /home/user1/.local/share/claude/versions/2.1.74` — v2.1.74 session (2026-06-29) | `Kf_()` and `WN()` at strings line 492301; constants at line 492298 | Error handling: `if(K==="ENOENT"\|\|K==="EISDIR")return null; if(K==="EACCES")Q("tengu_claude_md_permission_error",...)`. Constants: `L1="MEMORY.md"`, `$P=200` (line 492298); `ny4=5` (WN depth check: `if(q.has(A)\|\|O>=ny4)return[]`); `Xm=40000` (MAX_MEMORY_CHARACTER_COUNT); `QKT=3000`. Extension whitelist `Qy4=new Set([".md",".txt",...])` at line 492307, ~50+ types. MEMORY.md warning text confirmed verbatim. Empty-content guard: `if(!D\|\|!D.content.trim())return[]`. |
| E60 | B34 | Code | Binary analysis — `strings /home/user1/.local/share/claude/versions/2.1.74` — v2.1.74 session (2026-06-29) | `K2q()`, `S1()`, `ry4()` at lines 492298–492307 | `K2q()` assembly: `let q=Wq("tengu_paper_halyard",!1); for(let K of T){if(q&&(K.type==="Project"\|\|K.type==="Local"))continue;}`. User bypass: `q.push(...WN(J,"User",K,!0))` — always `includeExternal=true`. `ry4()` exclusion: `_p6.default.isMatch(O,R,{dot:!0})` (micromatch). Session disable: `a$()` checks `CLAUDE_CODE_DISABLE_CLAUDE_MDS\|\|sT(CLAUDE_CODE_SIMPLE)`. HTML strip: `Kp6` in `Rp6` module exports `{stripHtmlComments:()=>Kp6}`. GFM-off: `new $X({gfm:!1})` in `iy4()`. YAML frontmatter: `ly4()` processes `paths:` conditionals, content not passed to model. |
| E61 | B35 | Code | Binary analysis — `strings /home/user1/.local/share/claude/versions/2.1.74` — v2.1.74 session (2026-06-29) | `VfT()` adjacent to auto-memory functions `cy4`, `om6` | Full function: `function VfT(T){if(!Wq("tengu_coral_fern",!1))return[];let _=qw(R8()),q=Yz(),K=q?grep -rn... --include="*.md":${GR} with pattern=...; return["## Searching past context","","When looking for past context:","1. Search topic files...","2. Session transcript logs (last resort — large files, slow):","...","Use narrow search terms..."]}`. Default confirmed false via `Wq("tengu_coral_fern",!1)` — second arg is the fallback. Also confirmed: `function so(){return null}` — `QKT=3000` ultra-memory constant inoperative; all three `so()` call sites short-circuit on null in v2.1.74. |

---

### Statistical Summary

| Status | Count | IDs |
|--------|-------|-----|
| ✅ Confirmed | 20 | B1, B2, B3, B6, B7, B9, B10, B12, B13, B14, B16, B27, B28, B29, B30, B31, B32, B33, B34, B35 |
| 🎯 Observed | 14 | B4, B5, B8, B11, B15, B18, B19, B20, B21, B22, B23, B24, B25, B26 |
| ⚠️ Exception noted | 1 | B17 (self-contained except at context-compaction boundaries; < 0.2% violation rate) |
| ❓ Uncertain | 1 | B16h |

**Total behaviors:** 36 (B1–B35 + B16h sub-hypothesis; B16h shares B16's row index)
**Confirmed (≥90% certainty):** 20
**Lowest certainty:** B5 (60% — current session selection mechanism)
**Investigation priority:** B5 — can be confirmed by reading Claude Code changelog or source

| Test Tier | Count | IDs |
|-----------|-------|-----|
| VALIDATED | 12 | B1, B2, B6, B7, B9, B10, B12, B13, B14, B15, B17, B18 |
| VALIDATED† | 1 | B5 (distinct mtimes proven; mtime-as-selection-key unproven) |
| FLAG-VFY | 8 | B3, B4, B16, B19, B20, B21, B22, B24 |
| NEG-ONLY | 4 | B11, B23, B25, B26 |
| UNVERIFIED | 10 | B8, B27, B28, B29, B30, B31, B32, B33, B34, B35 |
| MEASURE | 1 | B16h (lim_it; runs by default in container) |

**Validation gap:** 12 of 36 behaviors are fully validated with behavioral assertions.

---

### Invalidation Tests

Each behavior instance has a corresponding invalidation test in `contract/claude_code/tests/behavior/`. Tests inspect real `~/.claude/` storage. If Claude Code changes behavior, the tests go RED.

| File | Behavior | Tier |
|------|----------|------|
| `b01_default_continues.rs` | B1 | VALIDATED |
| `b02_new_session.rs` | B2 | VALIDATED |
| `b03_print_flag.rs` | B3 | FLAG-VFY |
| `b04_continue_flag.rs` | B4 | FLAG-VFY |
| `b05_mtime_selection.rs` | B5 | VALIDATED† |
| `b06_session_accumulation.rs` | B6 | VALIDATED |
| `b07_agent_sessions.rs` | B7 | VALIDATED |
| `b08_zero_byte_init.rs` | B8 | UNVERIFIED |
| `b09_storage_path.rs` | B9 | VALIDATED |
| `b10_entry_threading.rs` | B10 | VALIDATED |
| `b11_auto_continue.rs` | B11 | NEG-ONLY |
| `b12_agent_session_id_is_parent.rs` | B12 | VALIDATED |
| `b13_subagent_directory_structure.rs` | B13 | VALIDATED |
| `b14_agent_meta_json.rs` | B14 | VALIDATED |
| `b15_agent_slug_field.rs` | B15 | VALIDATED |
| `b16_tools_disable.rs` | B16 | FLAG-VFY (parse-accept only; invocation-block requires lim_it) |
| `b17_parentuuid_self_contained.rs` | B17 | VALIDATED |
| `b18_no_cross_session_links.rs` | B18 | VALIDATED |
| `b19_resume_flag.rs` | B19 | FLAG-VFY |
| `b20_session_id_flag.rs` | B20 | FLAG-VFY |
| `b21_fork_session_flag.rs` | B21 | FLAG-VFY |
| `b22_no_session_persistence_flag.rs` | B22 | FLAG-VFY |
| `b23_session_dir_override.rs` | B23 | NEG-ONLY |
| `b24_from_pr_flag.rs` | B24 | FLAG-VFY |
| `b25_auto_compact_window.rs` | B25 | NEG-ONLY |
| `b26_autocompact_pct_override.rs` | B26 | NEG-ONLY |
| `b16h_tools_system_prompt.rs` | B16h | MEASURE (lim_it; runs by default in container) |
| `b27_agent_no_os_process.rs` | B27 | UNVERIFIED (no automated test yet) |
| `b28_bash_rtk_subprocess.rs` | B28 | UNVERIFIED (no automated test yet) |
| `b29_bash_claude_env.rs` | B29 | UNVERIFIED (no automated test yet) |
| `b30_subagent_context_inheritance.rs` | B30 | UNVERIFIED (no automated test yet) |
| `b31_subagent_tool_sets.rs` | B31 | UNVERIFIED (no automated test yet) |
| `b32_claudemd_at_ref_path_filter.rs` | B32 | UNVERIFIED (no automated test yet) |
| `b33_claudemd_loading_limits.rs` | B33 | UNVERIFIED (no automated test yet) |
| `b34_claudemd_content_pipeline.rs` | B34 | UNVERIFIED (no automated test yet) |
| `b35_automemory_search_context_flag.rs` | B35 | UNVERIFIED (no automated test yet) |

To run:
```bash
cd contract/claude_code && cargo nextest run --test behavior
```

### Type-Specific Requirements

All `behavior` doc instances must include:

1. **Title**: `# Behavior {ID}: {Short Name}` — using the B-prefix ID and a short descriptive name
2. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
3. **Behavior** (H3): The behavior statement, status/certainty/tier, and detail narrative
4. **Evidence** (H3): Subset of the master evidence table (rows from this readme that support this behavior)
5. **Cross-References** (H3): Flat table with `Type | File | Responsibility` columns

### Cross-Collection Dependencies

**This entity depends on**:
- `../storage/` — storage architecture concepts referenced in Continuation/Storage category behaviors
- `../jsonl/` — entry threading and `parentUuid` concepts referenced in B10, B17, B18
- `../taxonomy/` — Conversation Chain concept referenced in B18

**This entity consumed by**:
- `../../tests/behavior/` — invalidation test suite (one file per behavior)

- `../../../module/claude_runner_core/docs/claude_param/` — flag behavior cross-references
