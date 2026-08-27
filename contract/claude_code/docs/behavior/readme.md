# Behavior Doc Entity

### Scope

- **Purpose**: Catalog observed and confirmed external behaviors of the `claude` binary spanning session lifecycle, storage, runtime process model, subagent context, and tool availability.
- **Responsibility**: Master file for the `behavior` collection — lists all 38 behavior instances (B1–B37 + B16h), provides the shared evidence table (E1–E77), and links to invalidation test files. Refuted hypotheses are retained with their disconfirming evidence rather than deleted, so the record shows what was believed and why it was wrong.
- **In Scope**: Session continuation, flag semantics, agent layouts, entry threading, storage path encoding, cross-session relationship absence (conversation chain foundations); runtime process model (agent subagent identity, bash subprocess identity, env propagation); subagent context inheritance (CLAUDE.md injection, conversation absence, scope propagation); subagent tool availability per type (tool set differences, parent-exclusive tools); context loading (CLAUDE.md @-reference path filter, content pipeline transformations, silent failure and truncation modes); background task lifecycle (classifier model selection, idle-state reporting, exit-handoff survival, memory-pressure reaping, print-mode wait ceiling); subagent cache economics (per-subagent isolated cache prefix, 5-minute vs 1-hour cache TTL tier asymmetry).
- **Out of Scope**: Entry-level JSONL schema (→ [`../jsonl/`](../jsonl/readme.md)); storage directory architecture (→ [`../storage/`](../storage/readme.md)); filesystem paths (→ [`../filesystem/`](../filesystem/readme.md)); settings format (→ [`../settings/`](../settings/readme.md)); ancillary file formats (→ [`../format/`](../format/readme.md)); concept taxonomy (→ [`../taxonomy/`](../taxonomy/readme.md)).

### Overview Table

Adapted from hypothesis table format. Status reflects certainty of the observation, not investigation state. Certainty is based on source evidence — code inspection, observed output, or direct inference. All behaviors describe the external `claude` binary.

**Status legend:**
- ✅ Confirmed — source code or reproducible test confirms
- 🎯 Observed — seen in practice, mechanism inferred
- ❓ Uncertain — reasonable inference, unconfirmed
- ⚠️ Exception noted — holds in general, but with a documented and quantified exception class
- ❌ Refuted — disconfirming evidence found; the instance is retained to record the error, not deleted

**Test Tier legend:**
- `VALIDATED` — test asserts on real `~/.claude/` storage structure (hard `assert!` on fields/counts)
- `FLAG-VFY` — test verifies flag exists in `--help` or is accepted without parse error
- `NEG-ONLY` — test asserts env var is NOT explicitly rejected (cannot confirm acceptance vs silent ignore)
- `UNVERIFIED` — test has no `assert!`; logs observation only; never goes RED
- `MEASURE` — live API measurement; no pass/fail; runs by default in container where `~/.claude` is mounted (`lim_it_` prefix)
- `VALIDATED†` — test proves feasibility of mechanism but not that the binary uses it

**⚠️ NEG-ONLY reliability caveat.** A NEG-ONLY test asserts that the binary does not explicitly reject an env var. An env var the binary has **never heard of** is also not rejected, so the assertion passes byte-identically for a variable that is honored, one that is silently ignored, **and one that does not exist in the binary at all**. The tier therefore carries no evidence that the variable exists. B11 and B23 were both carried at 80–85% certainty on this basis and both turned out to be nonexistent (E72). Treat every remaining NEG-ONLY entry as unverified-existence until confirmed by a binary string scan or official documentation. Confirming existence is cheap: `grep -ac <VAR> ~/.local/share/claude/versions/<version>` with a known-present positive control and a fabricated negative control, as recorded in E72.

| ID | Behavior | Category | Status | Certainty | Tier | Since | Evidence |
|----|----------|----------|--------|-----------|------|-------|----------|
| [B1](001_b1_default_new_session.md) | `claude` binary defaults to NEW session; resuming requires explicit `--continue`/`-c`. `clr` wrapper inverts this default | Continuation | ✅ | 90% | VALIDATED | pre-v1.0 | E1, E2, E11, E47 |
| [B2](002_b2_new_session_creates_file.md) | Each invocation without `--continue` creates a new `.jsonl`; `--new-session` is a `clr` wrapper flag | Storage | ✅ | 95% | VALIDATED | pre-v1.0 | E1, E12, E47 |
| [B3](003_b3_print_orthogonal.md) | `-p`/`--print` controls output mode only; does not affect session selection | Flags | ✅ | 95% | FLAG-VFY | pre-v1.0 | E3, E13 |
| [B4](004_b4_continue_flag.md) | `-c`/`--continue` is explicit opt-in for resuming most recently modified session | Flags | 🎯 | 85% | FLAG-VFY | pre-v1.0 | E2, E14 |
| [B5](005_b5_mtime_selection.md) | `--continue` resumes the most recent session from a filtered candidate set (background, `-p`/SDK, and `/loop`-first sessions excluded; `-p --continue` excludes background only). Ordering key within that set is unconfirmed | Selection | ✅ filter / ❓ key | 95% / 55% | VALIDATED† | pre-v1.0 | E4, E15, E71 |
| [B6](006_b6_session_accumulation.md) | Sessions accumulate one file per independent invocation; never compacted or rotated | Storage | ✅ | 90% | VALIDATED | pre-v1.0 | E5, E16 |
| [B7](007_b7_agent_sessions_sibling.md) | Agent sessions are `agent-*.jsonl` siblings with `isSidechain: true` (flat layout) | Storage | ✅ | 95% | VALIDATED | pre-v1.0 | E6, E17 |
| [B8](008_b8_zero_byte_placeholder.md) | Claude Code creates zero-byte `.jsonl` placeholders on startup; remain if process crashes | Storage | 🎯 | 85% | UNVERIFIED | pre-v1.0 | E7, E18 |
| [B9](009_b9_storage_path_encoding.md) | Project sessions stored at `~/.claude/projects/{path-encoded}/`; **every non-alphanumeric** char → `-` (not `/` alone); names over 200 chars truncated + hashed. Encoding is lossy, not reversible | Storage | ✅ | 95% | UNVERIFIED | pre-v1.0; rule changed after 2026-07-16 | E8, E19, E74, E75 |
| [B10](010_b10_entry_threading.md) | Entries linked by `parentUuid`; root entry has `parentUuid: null` | Entries | ✅ | 95% | VALIDATED | pre-v1.0 | E9, E20 |
| [B11](011_b11_auto_continue_env.md) | ~~`CLAUDE_CODE_AUTO_CONTINUE` env var enables automated continuation mode~~ — **REFUTED**: 0 occurrences in the v2.1.220 binary; absent from official docs. Still exported by this workspace as a no-op | Flags | ❌ | 95% refuted | NEG-ONLY (insufficient) | refuted at v2.1.220 | E10, E21, E72 |
| [B12](012_b12_agent_session_id.md) | Agent JSONL entries carry `sessionId` equal to the parent session UUID | Families | ✅ | 95% | VALIDATED | pre-v1.0 | E22, E26 |
| [B13](013_b13_subagent_directory.md) | New-format agents stored at `{parent-uuid}/subagents/agent-{agentId}.jsonl` | Families | ✅ | 95% | VALIDATED | pre-v1.0 | E23, E27 |
| [B14](014_b14_agent_meta_json.md) | Agent `.meta.json` sidecars record spawn arguments: `agentType` always (7 known values) plus 9 optional fields (`spawnDepth`, `description`, `toolUseId`, `isFork`, `model`, `parentAgentId`, `stoppedByUser`, `worktreePath`, `worktreeBranch`); written flat in `subagents/` or nested in `subagents/workflows/wf_*/` | Families | ✅ | 95% | VALIDATED | pre-v1.0 | E24, E28, E77 |
| [B15](015_b15_agent_slug.md) | Agent entries carry a `slug` field shared by all agents of one parent | Families | 🎯 | 85% | VALIDATED | pre-v1.0 | E25, E29 |
| [B16](016_b16_tools_flag.md) | `--tools ""` disables all tool invocation; `--tools "default"` restores all tools | Flags | ✅ | 90% | FLAG-VFY | pre-v1.0 | E30, E31 |
| [B16h](016h_b16h_tools_system_prompt.md) | Tool definitions (~12k tokens) remain in assembled system prompt even with `--tools ""` | Flags | ❓ | 60% | MEASURE | pre-v1.0 | E32 |
| [B17](017_b17_parentuuid_self_contained.md) | `parentUuid` chain is self-contained within one session file (< 0.2% compaction exceptions) | Entries | ⚠️ | 85% | VALIDATED | pre-v1.0 | E33 |
| [B18](018_b18_no_cross_session_links.md) | No cross-session continuation metadata; first entry of new session has `parentUuid: null` | Continuation | 🎯 | 80% | VALIDATED | pre-v1.0 | E34 |
| [B19](019_b19_resume_flag.md) | `--resume`/`-r` resumes a specific prior session by UUID | Continuation | 🎯 | 85% | FLAG-VFY | pre-v1.0 | E35, E36 |
| [B20](020_b20_session_id_flag.md) | `--session-id <uuid>` assigns a deterministic UUID to the current session | Session | 🎯 | 80% | FLAG-VFY | pre-v1.0 | E37, E38 |
| [B21](021_b21_fork_session.md) | `--fork-session` creates a new session UUID when resuming; original unchanged | Continuation | 🎯 | 80% | FLAG-VFY | pre-v1.0 | E39, E40 |
| [B22](022_b22_no_session_persistence.md) | `--no-session-persistence` disables session disk writes; only works with `--print` mode | Storage | 🎯 | 85% | FLAG-VFY | pre-v1.0 | E41, E42 |
| [B23](023_b23_session_dir_override.md) | ~~`CLAUDE_CODE_SESSION_DIR` env var overrides session storage directory~~ — **REFUTED**: 0 occurrences in the v2.1.220 binary; absent from official docs. Real mechanism is `CLAUDE_CONFIG_DIR` | Storage | ❌ | 95% refuted | NEG-ONLY (insufficient) | refuted at v2.1.220 | E43, E44, E72, E73 |
| [B24](024_b24_from_pr.md) | `--from-pr [value]` resumes a session previously linked to a GitHub pull request | Continuation | 🎯 | 75% | FLAG-VFY | pre-v1.0 | E45, E46 |
| [B25](025_b25_auto_compact_window.md) | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` env var sets the effective token window for auto-compaction calculations; takes precedence over `/autocompact`, `--autocompact`, and the `autoCompactWindow` setting | Flags | 🎯 | 90% | NEG-ONLY (existence confirmed) | v2.1.75 | E48, E49, E76 |
| [B26](026_b26_autocompact_pct_override.md) | `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` env var overrides the compaction trigger as a percentage of the window; can only lower the threshold, never raise it; applies to subagents as well as main conversations | Flags | 🎯 | 85% | NEG-ONLY (existence confirmed) | v2.1.75 | E50, E51, E76 |
| [B27](027_b27_agent_no_os_process.md) | Agent tool subagents are not OS processes; run as API inference threads within the existing claude process — `pgrep` count unchanged before/during/after | Process Model | ✅ | 99% | UNVERIFIED | v2.1.74 | E52 |
| [B28](028_b28_bash_rtk_subprocess.md) | Each Bash tool call spawns a transient `rtk` proxy process (~5 MB, 4 FDs) that exits immediately; parent PID is gone before the next call | Process Model | ✅ | 95% | UNVERIFIED | v2.1.74 | E53 |
| [B29](029_b29_bash_claude_env.md) | All bash subprocesses inherit the full parent OS environment (107 vars, identical between parent and subagent) — including all `CLAUDE_*` vars, project vars, API keys, and desktop session vars | Process Model | ✅ | 99% | UNVERIFIED | v2.1.74 | E54, E56 |
| [B30](030_b30_subagent_context_inheritance.md) | Agent subagents receive full CLAUDE.md configuration (via system-reminder injection) but not the parent conversation history; scope variables (`SCOPE_DIR`/`SCOPE_READY`) are not inherited | Context | ✅ | 99% | UNVERIFIED | v2.1.74 | E55 |
| [B31](031_b31_subagent_tool_sets.md) | Subagent tool sets are narrower than the parent session (27 tools); general-purpose lacks Agent and 13 other session-management tools; Explore and Plan are identical; claude-code-guide uses static pre-loaded tools only | Tools | ✅ | 99% | UNVERIFIED | v2.1.74 | E57 |
| [B32](032_b32_claudemd_at_ref_path_filter.md) | The `iy4()` path filter silently rejects `$VAR`/`%VAR%` and other non-conforming @-reference prefixes; only `./`, `~/`, `/`, and `[a-zA-Z0-9._-]`-initial paths are accepted; `C9()` correctly expands `~/` to `os.homedir()` | Context Loading | ✅ | 99% | UNVERIFIED | v2.1.74 | E58 |
| [B33](033_b33_claudemd_loading_limits.md) | CLAUDE.md loading fails silently for ENOENT/EISDIR/EACCES and non-whitelisted extensions; `Xm=40,000` chars is a UI-warning-only threshold (file fully injected — ~10k tokens — but interactive status bar warns); hard limits: 200-line MEMORY.md cap (`$P`), 5-level @-include depth (`ny4`), 3,000-char ultra-memory (`QKT`) | Context Loading | ✅ | 99% | UNVERIFIED | v2.1.74 | E59 |
| [B34](034_b34_claudemd_content_pipeline.md) | HTML comments stripped (`Kp6`), YAML frontmatter processed as conditional globs not injected as content, GFM disabled in @-ref lexer; `tengu_paper_halyard` Statsig flag silently drops all Project/Local CLAUDE.md; User type always bypasses external-include dialog | Context Loading | ✅ | 99% | UNVERIFIED | v2.1.74 | E60 |
| [B35](035_b35_automemory_search_context_flag.md) | `tengu_coral_fern` Statsig flag (default false) gates a `## Searching past context` section in the auto-memory system prompt — provides grep commands for memory topic files and session JSONL transcripts; absent when flag is false | Auto-Memory | ✅ | 99% | UNVERIFIED | v2.1.74 | E61 |
| [B36](036_b36_background_task_lifecycle.md) | Five env vars gate the background-task lifecycle: `CLAUDE_CODE_BG_CLASSIFIER_MODEL` (classifier model override), `CLAUDE_CODE_BG_TASKS_REPORT_RUNNING` (idle-state reporting), `CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF` (exit-survival, excludes `agentId`-bearing jobs from `shells`), `CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP` (memory-pressure reaping), `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS` (print-mode wait ceiling, default 600000ms) | Background Tasks | ✅ | 85% | UNVERIFIED | ≤v2.1.197 | E62, E63, E64, E65, E66 |
| [B37](037_b37_subagent_cache_ttl.md) | Subagent API requests write to the 5-minute prompt-cache tier (`ephemeral_5m_input_tokens`) while the main conversation writes to the 1-hour tier on subscription; each subagent builds an isolated cache prefix from zero — no cache hits on its first call | Cache | ✅ | 99% | VALIDATED | ≤v2.1.197 | E67, E68, E69, E70 |

---

### Evidence Table

Evidence items are shared across behaviors (M:N relationship). Each item may support multiple behaviors.

**Location anchors are `grep` patterns, not line numbers.** A line number is a citation that
rots silently: the file keeps compiling, the doc keeps rendering, and the row keeps *looking*
cited while pointing at unrelated code. All four repo-source rows carried stale anchors when
this was audited — E1 and E3 cited `claude_runner/src/main.rs` lines 83/85/124, but `main.rs`
had shrunk to a 10-line shim delegating to `run_cli()`, and the referenced help text had moved
to `cli/help.rs`; E2 cited `command.rs:600`, but the file became the `command/` directory
module and the code itself changed from `parts.push("-c")` to
`tokens.push( ArgToken::Plain( … ) )`. Only E10 was still accurate, and only by luck.

A pattern anchor fails loudly instead — `grep` returns nothing, and the row is visibly
unverifiable rather than quietly wrong. Where an old line reference was replaced, the row
keeps it in parentheses so the history is not lost. Re-check every repo-source row at once:

```bash
cd contract/claude_code/docs/behavior
grep -oE '`\.\./[^`]+\.rs`' readme.md | tr -d '`' | sort -u | while read -r f; do
  [ -e "$f" ] || echo "MISSING: $f"
done
```

Binary-analysis rows (E58–E66) are a separate case: their `strings`-output line numbers are
valid only for the exact binary version named in the row, and are not expected to survive an
upgrade. Each such row names its version for that reason.

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E1 | B1, B2 | Code | `../../../../module/claude_runner/src/cli/help.rs` | `grep -n '"--new-session"'` (was `main.rs:85`) | `OptionEntry { name : "--new-session", desc : "Start a new session (default: continues previous)" }` — `clr` wrapper help text; confirms wrapper default is continuation (not the `claude` binary native default) |
| E2 | B1, B4 | Code | `../../../../module/claude_runner_core/src/command/mod.rs` | `grep -n 'continue_conversation {'` (was `command.rs:600`) | `if self.continue_conversation { tokens.push( ArgToken::Plain( "-c".to_string() ) ) }` — `-c` is a builder option wrapping the native flag |
| E3 | B3 | Code | `../../../../module/claude_runner/src/cli/help.rs`, `../../../../module/claude_runner/src/cli/parse.rs` | `grep -n '"--print"'` (was `main.rs:83,124`) | `OptionEntry { name : "-p, --print", desc : "Non-interactive mode (capture and print output)" }` in `help.rs`; the `"-p" \| "--print" =>` branch in `parse.rs` sets print-only; no session flag change |
| E4 | B5 | Inference | Storage observation | `~/.claude/projects/*/` | Multiple `.jsonl` files in one project; `--continue` must pick one; mtime is the only per-file ordering signal available without metadata |
| E5 | B6 | Observation | Live storage | `~/.claude/projects/…/-commit/` | 25 `.jsonl` files observed in one project directory from repeated sessions |
| E6 | B7 | Observation | Live storage | `~/.claude/projects/*/agent-*.jsonl` | Agent session files observed as siblings of main sessions; entries contain `"isSidechain":true` |
| E7 | B8 | Observation | Live storage | `~/.claude/projects/*/` | Zero-byte `.jsonl` files observed in project directories alongside non-empty sessions |
| E8 | B9 | Observation | Live storage | `~/.claude/projects/` | Project directory names are the working directory path with separators replaced by `-`. Originally read as a `/`→`-` rule; superseded by E74/E75, which show the current rule converts *every* non-alphanumeric character. The observation was consistent with both rules because the sampled paths happened to contain no underscores, dots, or spaces. |
| E9 | B10 | Doc | `../jsonl/009_threading_model.md` | Threading model | `parentUuid` links each entry to its parent; null on first entry of a thread |
| E10 | B11 | Code | `../../../../module/claude_runner_core/src/command/mod.rs` | `grep -n CLAUDE_CODE_AUTO_CONTINUE` | `pairs.push( ( "CLAUDE_CODE_AUTO_CONTINUE", auto_continue.to_string() ) )` — env var exported before spawning `claude`. Proves the workspace *sets* the variable; carries no information about whether the binary *reads* it. Location corrected 2026-08-27: previously cited as `src/command.rs` lines 647–648, a path and line range that no longer exist. |
| E11 | B1 | Test | `../../tests/behavior/b01_default_continues.rs` | `b1_resumable_session_exists_in_real_storage` | At least one non-empty non-agent session exists in real `~/.claude/` storage — prerequisite for default continuation |
| E12 | B2 | Test | `../../tests/behavior/b02_new_session.rs` | `b2_multiple_session_files_exist_in_real_project` | At least one project in real `~/.claude/` storage has 2+ non-empty non-agent `.jsonl` files — evidence of per-session file creation |
| E13 | B3 | Test | `../../tests/behavior/b03_print_flag.rs` | `b3_print_flag_documented_as_output_mode` | `claude --help` documents `-p` / `--print` as output mode |
| E14 | B4 | Test | `../../tests/behavior/b04_continue_flag.rs` | `b4_continue_flag_documented_in_help` | `claude --help` documents `-c` / `--continue` flag |
| E15 | B5 | Test | `../../tests/behavior/b05_mtime_selection.rs` | `b5_real_sessions_have_distinct_mtimes` | Real project with 2+ sessions has distinct mtimes — mtime ordering is possible |
| E16 | B6 | Test | `../../tests/behavior/b06_session_accumulation.rs` | `b6_sessions_accumulate_in_real_project` | Real project directory contains 5+ `.jsonl` files — higher threshold than B2 to confirm long-term accumulation without rotation |
| E17 | B7 | Test | `../../tests/behavior/b07_agent_sessions.rs` | `b7_real_agent_session_has_issidechain_true` | Real `agent-*.jsonl` file contains `"isSidechain":true` in first entry |
| E18 | B8 | Observation | `../../tests/behavior/b08_zero_byte_init.rs` | `b8_zero_byte_jsonl_exists_in_real_storage` | Zero-byte `.jsonl` files observed in real `~/.claude/` storage (test logs observation, does not assert) |
| E19 | B9 | Test | `../../tests/behavior/b09_storage_path.rs` | `b9_project_dir_names_follow_encoding_convention` | Asserts only that at least one project directory name starts with `-`; the `-`→`/` round-trip is best-effort and the test passes even when every decode fails. Does not assert the character class, so it cannot detect an encoding-rule change. |
| E20 | B10 | Test | `../../tests/behavior/b10_entry_threading.rs` | `b10_first_entry_has_null_parent_uuid`, `b10_subsequent_entries_have_non_null_parent_uuid` | First conversation entry has `parentUuid:null`; second has non-null `parentUuid` referencing first |
| E21 | B11 | Test | `../../tests/behavior/b11_auto_continue.rs` | `b11_auto_continue_env_var_recognized` | Binary does not print `CLAUDE_CODE_AUTO_CONTINUE` in stderr when env var is set — negative assertion; passes identically for a variable absent from the binary, which is why it did not catch this refutation |
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
| E43 | B23 | Doc | `../param/057_session_dir.md` | Description | Documents `CLAUDE_CODE_SESSION_DIR` env var that overrides session storage directory. This is a *self-citation within this collection*, not external corroboration — the param doc and B23 were written from the same unverified assumption, so it never constituted independent evidence. Superseded by E72/E73. |
| E44 | B23 | Test | `../../tests/behavior/b23_session_dir_override.rs` | `b23_session_dir_env_var_not_rejected` | Binary does not explicitly reject `CLAUDE_CODE_SESSION_DIR` env var at startup — passes identically for a nonexistent variable, which is why it did not catch this refutation |
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
| E58 | B32 | Code | Binary analysis — `strings /home/alice/.local/share/claude/versions/2.1.74` — v2.1.74 session (2026-06-29) | `iy4()` at strings line 492301; `C9()` at binary offset 108,423,272 | `iy4()` path filter: `j.startsWith("./") \|\| j.startsWith("~/") \|\| (j.startsWith("/") && j !== "/") \|\| (!j.startsWith("@") && !j.match(/^[#%^&*()]+/) && j.match(/^[a-zA-Z0-9._-]/))`. `C9()` resolver: `if(K==="~")return Uo_.homedir().normalize("NFC"); if(K.startsWith("~/"))return SZ.join(Uo_.homedir(),K.slice(2)).normalize("NFC"); if(SZ.isAbsolute(O))return SZ.normalize(O).normalize("NFC"); return SZ.resolve(q,O).normalize("NFC")`. Regex: `/(?:^|\s)@((?:[^\s\\]|\\ )+)/g`. Code/codespan skip: `if(H.type==="code"\|\|H.type==="codespan")continue`. Fragment strip: `let w=j.indexOf("#"); if(w!==-1)j=j.substring(0,w)`. |
| E59 | B33 | Code | Binary analysis — `strings /home/alice/.local/share/claude/versions/2.1.74` — v2.1.74 session (2026-06-29) | `Kf_()` and `WN()` at strings line 492301; constants at line 492298 | Error handling: `if(K==="ENOENT"\|\|K==="EISDIR")return null; if(K==="EACCES")Q("tengu_claude_md_permission_error",...)`. Constants: `L1="MEMORY.md"`, `$P=200` (line 492298); `ny4=5` (WN depth check: `if(q.has(A)\|\|O>=ny4)return[]`); `Xm=40000` (MAX_MEMORY_CHARACTER_COUNT); `QKT=3000`. Extension whitelist `Qy4=new Set([".md",".txt",...])` at line 492307, ~50+ types. MEMORY.md warning text confirmed verbatim. Empty-content guard: `if(!D\|\|!D.content.trim())return[]`. |
| E60 | B34 | Code | Binary analysis — `strings /home/alice/.local/share/claude/versions/2.1.74` — v2.1.74 session (2026-06-29) | `K2q()`, `S1()`, `ry4()` at lines 492298–492307 | `K2q()` assembly: `let q=Wq("tengu_paper_halyard",!1); for(let K of T){if(q&&(K.type==="Project"\|\|K.type==="Local"))continue;}`. User bypass: `q.push(...WN(J,"User",K,!0))` — always `includeExternal=true`. `ry4()` exclusion: `_p6.default.isMatch(O,R,{dot:!0})` (micromatch). Session disable: `a$()` checks `CLAUDE_CODE_DISABLE_CLAUDE_MDS\|\|sT(CLAUDE_CODE_SIMPLE)`. HTML strip: `Kp6` in `Rp6` module exports `{stripHtmlComments:()=>Kp6}`. GFM-off: `new $X({gfm:!1})` in `iy4()`. YAML frontmatter: `ly4()` processes `paths:` conditionals, content not passed to model. |
| E61 | B35 | Code | Binary analysis — `strings /home/alice/.local/share/claude/versions/2.1.74` — v2.1.74 session (2026-06-29) | `VfT()` adjacent to auto-memory functions `cy4`, `om6` | Full function: `function VfT(T){if(!Wq("tengu_coral_fern",!1))return[];let _=qw(R8()),q=Yz(),K=q?grep -rn... --include="*.md":${GR} with pattern=...; return["## Searching past context","","When looking for past context:","1. Search topic files...","2. Session transcript logs (last resort — large files, slow):","...","Use narrow search terms..."]}`. Default confirmed false via `Wq("tengu_coral_fern",!1)` — second arg is the fallback. Also confirmed: `function so(){return null}` — `QKT=3000` ultra-memory constant inoperative; all three `so()` call sites short-circuit on null in v2.1.74. |
| E62 | B36 | Code | Binary analysis — `strings -a -n 8` + `grep -aoP` on `~/.local/share/claude/versions/2.1.197` — this session (2026-07-07) | `strings`-output line 122948 (bare name) and line 272025 (export map) | `CLAUDE_CODE_BG_CLASSIFIER_MODEL:()=>KDu` in module export map, adjacent to `ANTHROPIC_SMALL_FAST_MODEL`, `CLAUDE_CODE_AUTO_MODE_MODEL`, `CLAUDE_CODE_ALWAYS_ENABLE_EFFORT`; schema declaration `KDu=Ue.str()` found in the same settings-schema object as `JDu=Ue.bool()`, `QDu=Ue.bool()`, etc. No direct `KDu(...)` call site found. |
| E63 | B36 | Code | Binary analysis — same binary/method — this session (2026-07-07) | `strings`-output line 296515/296542, functions `S3c`/`b3c` | `function b3c({inputClosed:e,runningTasks:t}){return e&&t.some((n)=>XX(n)&&wv(n))}function S3c({inputClosed:e,currentState:t,hasRunningBgTasks:n}){if(n&&Fe.CLAUDE_CODE_BG_TASKS_REPORT_RUNNING)return!1;return!e&&t==="running"}` — full function bodies recovered verbatim. |
| E64 | B36 | Code | Binary analysis — same binary/method — this session (2026-07-07) | `strings`-output line 274061, functions `_Ha`/`bHa` | `function _Ha(e){if(!yi()\|\|!Fe.CLAUDE_JOB_DIR\|\|Fe.CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF)return{shells:[],workflows:[]};let t=nEe(e),n=Object.values(e);return{shells:n.filter((r)=>Tbo(r,t)&&r.agentId===void 0),workflows:n.filter((r)=>Ebo(r,t))}}function bHa({shells:e,workflows:t}){let n=Fe.CLAUDE_JOB_DIR;for(let o of t)o.abortController...` (truncated by strings extraction after `abortController`). Confirms `agentId===void 0` as an explicit filter condition on the `shells` survivor set. |
| E65 | B36 | Code + Doc | Binary analysis (same method, this session, 2026-07-07) + official changelog `../version/091_v2_1_193.md` | `strings`-output line 277007, function `u0l`; changelog entry v2.1.193 | `function u0l(e,t,n,r,o,s){Pve(s,\`bash:${e}\`,n);let i;if(s===void 0&&!xr()&&!Fe.CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP){let a=()=>{let l=n.get(e);if(l?.status!=="running"\|\|l.notified\|\|Date.now()-NA()<Exm\|\|gEr()\|\|yKe(n.all()))return;Ie("task_local_shell_pressure_reap"),tYt(e,t,"killed",void 0,n,r,o,s),nve(e,n)};process.on("memoryPressure",a),i=()=>process.off("memory...` (truncated). Changelog: "Added automatic memory-pressure reaping for idle background shell commands (disable with `CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP=1`)" — the only one of the five vars officially documented. |
| E66 | B36 | Code | Binary analysis — same binary/method — this session (2026-07-07) | `strings`-output line 296515/296542, functions `E3c`/`v3c`/`w3c`; constants `KFf`, `tZo` | `function E3c(){return Fe.CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS??KFf}` plus full `v3c()`/`w3c()` bodies (see [036_b36_background_task_lifecycle.md](036_b36_background_task_lifecycle.md)). Constants independently confirmed via `grep -aoP "(?<![a-zA-Z0-9_])KFf\s*=\s*[0-9]+"` → `KFf=600000`; same technique → `tZo=5000`. |
| E67 | B37 | Doc | Official Claude Code documentation (code.claude.com/docs/en/prompt-caching) | "Subagents and the cache" section | "A subagent starts its own conversation with its own system prompt and tool set. It builds its own cache, starting with no cache hits on its first call." And: "Subagents use the five-minute TTL even on a subscription, since the automatic one-hour TTL applies to the main conversation." Forks are the documented exception — a fork inherits the parent conversation's cache. Timer semantics: "Each request that hits the cache resets the timer." |
| E68 | B37 | Observation | Live session JSONL — session `feed0011` (2026-07-25, v2.1.197) | Main session file vs `subagents/agent-*.jsonl` siblings | Main-conversation assistant entries: `"cache_creation":{"ephemeral_5m_input_tokens":0,"ephemeral_1h_input_tokens":4908}` — 1-hour tier only. All 13 subagent transcripts from the same session (`isSidechain: true`): 5-minute tier only (`ephemeral_1h` = 0), per-agent first-call prefix writes of 42,884–72,407 tokens (769,900 cache-write tokens total for prefixes the parent already held cached). |
| E69 | B37 | Doc | Anthropic platform documentation (docs.anthropic.com — prompt caching pricing) + code.claude.com/docs/en/costs | Pricing multipliers; TTL policy | Cache writes bill 1.25x base input (5-minute TTL) / 2x (1-hour TTL); cache reads bill 0.1x. Costs doc: the 1-hour TTL applies to the main conversation on subscription and drops to 5 minutes when drawing on extra usage credits; `/usage` attributes a distinct "subagents" category and flags "cache misses" when ≥10% of recent usage. |
| E70 | B37 | Test | `../../tests/behavior/b37_subagent_cache_ttl.rs` | `b37_plain_agent_transcripts_never_write_1h_tier`, `b37_main_sessions_write_1h_tier_on_subscription` | Full-storage scan (2026-07-26): 12,861 plain-hex non-fork agent transcripts, 742,911 `cache_creation` entries, 740,976 five-minute writes, zero 1-hour writes — hard assert. Excluded 20 fork agents and 1,016 typed-prefix system sidechains, which inherit the parent conversation's tier (18 forks and 1,014 sidechains carry 1-hour writes). Main-session 1-hour write confirmed on the same machine. |
| E71 | B5 | Doc | Official Claude Code documentation (code.claude.com/docs/en/sessions § Resume a session) | `--continue` row and following paragraph | "`claude --continue` — Resumes the most recent interactive session in the current directory." And: "Claude Code leaves sessions created with `claude -p` or the Agent SDK out of the session picker and out of `claude --continue`… With `claude --continue`, Claude Code also skips background sessions and sessions whose first prompt was `/loop`. When you run `claude -p --continue`, Claude Code includes `-p`, SDK, and `/loop` sessions and still skips background sessions." No sort key is stated. |
| E72 | B23, B11 | Experiment | Binary string scan — `grep -ac <VAR> ~/.local/share/claude/versions/2.1.220` (2026-08-27) | v2.1.220 native binary, 271,825,824 bytes | Occurrence counts: `CLAUDE_CODE_SESSION_DIR` = 0, `CLAUDE_CODE_AUTO_CONTINUE` = 0. Positive controls in the same scan: `CLAUDE_CONFIG_DIR` = 28, `CLAUDE_CODE_SKIP_PROMPT_HISTORY` = 9, `CLAUDE_CODE_ENTRYPOINT` = 41, `CLAUDECODE` = 20, `cleanupPeriodDays` = 12. Negative control `TOTALLY_FAKE_VAR_XYZ` = 0. Method control: `CLAUDE_CODE_PROJECT_DIR_NAME` = 0, which is the expected result since official docs state it requires v2.1.234 — confirming the scan reports absence correctly rather than under-matching. |
| E73 | B23 | Doc | Official Claude Code documentation (code.claude.com/docs/en/sessions § Where transcripts are stored) | Configuration table | Lists the supported controls for transcript location and retention: `CLAUDE_CONFIG_DIR` to "Move storage off `~/.claude`", `CLAUDE_CODE_PROJECT_DIR_NAME` to name the project directory (v2.1.234+), `cleanupPeriodDays` for the 30-day retention, `CLAUDE_CODE_SKIP_PROMPT_HISTORY` to suppress transcript writes in all modes, and `--no-session-persistence` for one non-interactive run. `CLAUDE_CODE_SESSION_DIR` appears nowhere in official documentation. |
| E74 | B9 | Doc | Official Claude Code documentation (code.claude.com/docs/en/sessions § Where transcripts are stored) | Storage path paragraph | "By default, Claude Code stores transcripts as JSONL at `~/.claude/projects/<project>/<session-id>.jsonl`, where `<project>` is your working directory path with non-alphanumeric characters replaced by `-`. For a working directory whose converted name exceeds 200 characters, Claude Code truncates the name to 200 characters and appends a hash of the full path, so the directory name stays within filesystem limits." |
| E75 | B9 | Experiment | Live storage survey — `~/.claude/projects/` (2026-08-27, v2.1.220) | 978 project directories | Character census: 0 directory names contain a space, 0 contain a dot, 8 contain an underscore. The 8 underscore-preserving names were last written 2026-06-29 → 2026-07-16 (legacy rule); every current-era name converts underscores. Direct current-version confirmation: this session's cwd `/home/user1/pro/lib/yrd_core/family_ai/claude_runner/module/claude_runner/docs` writes its transcript to `-home-user1-pro-lib-yrd-core-family-ai-claude-runner-module-claude-runner-docs`, converting `yrd_core`→`yrd-core`, `family_ai`→`family-ai`, `claude_runner`→`claude-runner`. Forward-encoding `sed 's/[^a-zA-Z0-9]/-/g'` over real source directories reproduces existing project directory names exactly. Longest name observed: 168 chars, so the 200-char truncation rule is unexercised on this machine. |
| E76 | B25, B26 | Experiment | Binary string scan — `grep -ac <VAR> ~/.local/share/claude/versions/2.1.220` (2026-08-27) | v2.1.220 native binary | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` = 14, `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` = 6 — both present. Same scan, same controls as E72, which returned 0 for the two refuted variables: positive control `CLAUDE_CONFIG_DIR` = 28, negative control `TOTALLY_FAKE_VAR_XYZ` = 0. Establishes that these two NEG-ONLY behaviors describe variables the binary actually contains, which the NEG-ONLY test itself cannot show. |
| E77 | B14 | Experiment | Live storage census (2026-08-27, v2.1.220) | `~/.claude/projects/**/subagents/**/*.meta.json` | 16713 sidecars carrying `agentType`: general-purpose 11088, Explore 5217, workflow-subagent 269, fork 119, Plan 10, claude-code-guide 6, claude 4. Layout split: 16384 flat in `subagents/`, 329 nested in `subagents/workflows/wf_*/` (269 workflow-subagent + 60 Explore). Full key census finds ten fields, not two: `agentType` 16713, `spawnDepth` 16095 (values 1-4), `description` 15769, `toolUseId` 15766, `isFork` 119, `model` 76, `parentAgentId` 11, `stoppedByUser` 8, `worktreePath` 3, `worktreeBranch` 3 |

**Provenance caveat — E58 through E61.** These four rows cite `strings /home/alice/.local/share/claude/versions/2.1.74`. Neither the path nor the version is reachable from this repository's environment: `/home/alice` does not exist, and the only installed version is 2.1.220. They are therefore **not reproducible as written** — re-running them requires substituting the current `$HOME` and an installed version, and the offsets, `strings`-output line numbers, and minified identifiers (`iy4`, `C9`, `Kf_`, `K2q`, `VfT`, …) are specific to the 2.1.74 build and will not survive a version change. The findings are retained because the recovered function bodies are quoted verbatim and remain the best available record, but any claim that depends on them should be re-derived against the installed binary before being relied on. The same caveat applies to E62–E66, which cite v2.1.197 — also no longer installed, though at least under a `$HOME` that does exist.

---

### Statistical Summary

| Status | Count | IDs |
|--------|-------|-----|
| ✅ Confirmed | 22 | B1, B2, B3, B6, B7, B9, B10, B12, B13, B14, B16, B27, B28, B29, B30, B31, B32, B33, B34, B35, B36, B37 |
| ✅ / ❓ Split | 1 | B5 (candidate filter confirmed at 95%; ordering key uncertain at 55%) |
| 🎯 Observed | 11 | B4, B8, B15, B18, B19, B20, B21, B22, B24, B25, B26 |
| ⚠️ Exception noted | 1 | B17 (self-contained except at context-compaction boundaries; < 0.2% violation rate) |
| ❓ Uncertain | 1 | B16h |
| ❌ Refuted | 2 | B11, B23 (both `CLAUDE_CODE_*` env vars absent from the v2.1.220 binary; both were NEG-ONLY) |

**Total behaviors:** 38 (B1–B37 + B16h sub-hypothesis; B16h shares B16's row index) — 22 + 1 + 11 + 1 + 1 + 2 = 38.
**Confirmed (≥90% certainty):** 21 (B36 is Confirmed status at 85% certainty — below the 90% threshold, included in the Confirmed row above by evidence type but excluded from this ≥90% count)
**Lowest certainty:** B5 ordering key (55% — which field orders the filtered candidate set)
**Investigation priority:** the 4 remaining NEG-ONLY entries. Two of the six original NEG-ONLY behaviors (B11, B23) were refuted outright once scanned against the binary; B25 and B26 survived the same scan (`CLAUDE_CODE_AUTO_COMPACT_WINDOW` = 14, `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` = 6) and are additionally documented officially. That is a 33% refutation rate for the tier — every future NEG-ONLY entry should be scanned before it is assigned a certainty above 50%.

| Test Tier | Count | IDs |
|-----------|-------|-----|
| VALIDATED | 12 | B1, B2, B6, B7, B10, B12, B13, B14, B15, B17, B18, B37 |
| VALIDATED† | 1 | B5 (distinct mtimes proven; mtime-as-selection-key unproven) |
| FLAG-VFY | 8 | B3, B4, B16, B19, B20, B21, B22, B24 |
| NEG-ONLY | 4 | B11, B23 (both refuted — the tier could not detect it), B25, B26 |
| UNVERIFIED | 12 | B8, B9, B27, B28, B29, B30, B31, B32, B33, B34, B35, B36 |
| MEASURE | 1 | B16h (lim_it; runs by default in container) |

Total: 12 + 1 + 8 + 4 + 12 + 1 = 38.

**Validation gap:** 12 of 38 behaviors are fully validated with behavioral assertions. B9 moved from VALIDATED to UNVERIFIED in this revision: its test asserts a leading-character convention, not the encoding rule the behavior claims, so it could not detect the rule change that E74/E75 document. A test tiered VALIDATED must assert the behavior statement itself — asserting some weaker property of the same data is what let B9 carry a false tier.

---

### Invalidation Tests

Behavior instances B1–B26, B16h, and B37 have an invalidation test in `contract/claude_code/tests/behavior/`. Tests inspect real `~/.claude/` storage. If Claude Code changes behavior, the tests go RED — subject to the tier's own strength: only `VALIDATED` tests assert the behavior statement itself.

**B27–B36 have no test file.** 28 test files exist (`ls contract/claude_code/tests/behavior/*.rs`); before this revision this table listed ten additional filenames — `b27_agent_no_os_process.rs` through `b36_background_task_lifecycle.rs` — none of which exist on disk. They are listed below as *absent* rather than deleted, since each names a real coverage gap worth filling. All ten behaviors rest on one-off experiments and binary analysis recorded in E52–E66, with nothing that goes RED on regression.

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
| `b09_storage_path.rs` | B9 | UNVERIFIED (asserts a leading-`-` convention, not the encoding rule — see B9) |
| `b10_entry_threading.rs` | B10 | VALIDATED |
| `b11_auto_continue.rs` | B11 | NEG-ONLY (behavior refuted; the test still passes — see the NEG-ONLY caveat) |
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
| `b23_session_dir_override.rs` | B23 | NEG-ONLY (behavior refuted; the test still passes — see the NEG-ONLY caveat) |
| `b24_from_pr_flag.rs` | B24 | FLAG-VFY |
| `b25_auto_compact_window.rs` | B25 | NEG-ONLY |
| `b26_autocompact_pct_override.rs` | B26 | NEG-ONLY |
| `b16h_tools_system_prompt.rs` | B16h | MEASURE (lim_it; runs by default in container) |
| `b37_subagent_cache_ttl.rs` | B37 | VALIDATED |
| *(absent)* `b27_agent_no_os_process.rs` | B27 | **No test file** — would assert `pgrep -a claude` count is unchanged across an Agent dispatch |
| *(absent)* `b28_bash_rtk_subprocess.rs` | B28 | **No test file** — would assert `/proc/self/status` reports `Name: rtk` inside a Bash tool call |
| *(absent)* `b29_bash_claude_env.rs` | B29 | **No test file** — would diff `/proc/self/environ` between parent and subagent |
| *(absent)* `b30_subagent_context_inheritance.rs` | B30 | **No test file** — requires live subagent dispatch; not reachable from a test harness |
| *(absent)* `b31_subagent_tool_sets.rs` | B31 | **No test file** — requires live subagent dispatch; not reachable from a test harness |
| *(absent)* `b32_claudemd_at_ref_path_filter.rs` | B32 | **No test file** — would re-derive the `iy4()` path filter against the installed binary |
| *(absent)* `b33_claudemd_loading_limits.rs` | B33 | **No test file** — would assert the `$P`/`ny4`/`Xm` constants against the installed binary |
| *(absent)* `b34_claudemd_content_pipeline.rs` | B34 | **No test file** — would assert the pipeline transformations against the installed binary |
| *(absent)* `b35_automemory_search_context_flag.rs` | B35 | **No test file** — would assert the `tengu_coral_fern` default against the installed binary |
| *(absent)* `b36_background_task_lifecycle.rs` | B36 | **No test file** — would assert the five background-task env vars are present in the installed binary |

The six binary-analysis rows (B32–B36, and the existence half of every NEG-ONLY behavior) are all testable by the same cheap mechanism that refuted B11 and B23: scan the installed binary for the identifier and assert a non-zero count, with a fabricated negative control to prove the scan discriminates. That does not confirm semantics, but it does convert "documented from a build we no longer have" into something that goes RED when the identifier disappears.

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
