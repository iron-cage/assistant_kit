# Behavior

### Scope

- **Purpose**: Catalog observed and confirmed external behaviors of the `claude` binary relevant to session lifecycle and storage.
- **Responsibility**: Master file for the `behavior` collection — lists all 25 behavior instances (B1–B24 + B16h), provides the shared evidence table (E1–E46), and links to invalidation test files.
- **In Scope**: Session continuation, flag semantics, agent layouts, entry threading, storage path encoding, cross-session relationship absence (conversation chain foundations).
- **Out of Scope**: Entry-level JSONL schema (→ [`../jsonl/`](../jsonl/readme.md)); storage directory architecture (→ [`../storage/`](../storage/readme.md)); filesystem paths (→ [`../filesystem/`](../filesystem/readme.md)); settings format (→ [`../settings/`](../settings/readme.md)); ancillary file formats (→ [`../formats/`](../formats/readme.md)); concept taxonomy (→ [`../taxonomy/`](../taxonomy/readme.md)).

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

| ID | Behavior | Category | Status | Certainty | Tier | Evidence |
|----|----------|----------|--------|-----------|------|----------|
| [B1](001_b1_default_new_session.md) | `claude` binary defaults to NEW session; resuming requires explicit `--continue`/`-c`. `clr` wrapper inverts this default | Continuation | ✅ | 90% | VALIDATED | E1, E2, E11, E47 |
| [B2](002_b2_new_session_creates_file.md) | Each invocation without `--continue` creates a new `.jsonl`; `--new-session` is a `clr` wrapper flag | Storage | ✅ | 95% | VALIDATED | E1, E12, E47 |
| [B3](003_b3_print_orthogonal.md) | `-p`/`--print` controls output mode only; does not affect session selection | Flags | ✅ | 95% | FLAG-VFY | E3, E13 |
| [B4](004_b4_continue_flag.md) | `-c`/`--continue` is explicit opt-in for resuming most recently modified session | Flags | 🎯 | 85% | FLAG-VFY | E2, E14 |
| [B5](005_b5_mtime_selection.md) | "Current" session resumed by `--continue` is the most recently modified `.jsonl` (mtime) | Selection | 🎯 | 60% | VALIDATED† | E4, E15 |
| [B6](006_b6_session_accumulation.md) | Sessions accumulate one file per independent invocation; never compacted or rotated | Storage | ✅ | 90% | VALIDATED | E5, E16 |
| [B7](007_b7_agent_sessions_sibling.md) | Agent sessions are `agent-*.jsonl` siblings with `isSidechain: true` (flat layout) | Storage | ✅ | 95% | VALIDATED | E6, E17 |
| [B8](008_b8_zero_byte_placeholder.md) | Claude Code creates zero-byte `.jsonl` placeholders on startup; remain if process crashes | Storage | 🎯 | 85% | UNVERIFIED | E7, E18 |
| [B9](009_b9_storage_path_encoding.md) | Project sessions stored at `~/.claude/projects/{path-encoded}/`; `/` → `-` | Storage | ✅ | 95% | VALIDATED | E8, E19 |
| [B10](010_b10_entry_threading.md) | Entries linked by `parentUuid`; root entry has `parentUuid: null` | Entries | ✅ | 95% | VALIDATED | E9, E20 |
| [B11](011_b11_auto_continue_env.md) | `CLAUDE_CODE_AUTO_CONTINUE` env var enables automated continuation mode | Flags | 🎯 | 85% | NEG-ONLY | E10, E21 |
| [B12](012_b12_agent_session_id.md) | Agent JSONL entries carry `sessionId` equal to the parent session UUID | Families | ✅ | 95% | VALIDATED | E22, E26 |
| [B13](013_b13_subagent_directory.md) | New-format agents stored at `{parent-uuid}/subagents/agent-{agentId}.jsonl` | Families | ✅ | 95% | VALIDATED | E23, E27 |
| [B14](014_b14_agent_meta_json.md) | Agent `.meta.json` sidecars contain `agentType` and optional `description` | Families | ✅ | 90% | VALIDATED | E24, E28 |
| [B15](015_b15_agent_slug.md) | Agent entries carry a `slug` field shared by all agents of one parent | Families | 🎯 | 85% | VALIDATED | E25, E29 |
| [B16](016_b16_tools_flag.md) | `--tools ""` disables all tool invocation; `--tools "default"` restores all tools | Flags | ✅ | 90% | FLAG-VFY | E30, E31 |
| [B16h](016h_b16h_tools_system_prompt.md) | Tool definitions (~12k tokens) remain in assembled system prompt even with `--tools ""` | Flags | ❓ | 60% | MEASURE | E32 |
| [B17](017_b17_parentuuid_self_contained.md) | `parentUuid` chain is self-contained within one session file (< 0.2% compaction exceptions) | Entries | 🎯 | 85% | VALIDATED | E33 |
| [B18](018_b18_no_cross_session_links.md) | No cross-session continuation metadata; first entry of new session has `parentUuid: null` | Continuation | 🎯 | 80% | VALIDATED | E34 |
| [B19](019_b19_resume_flag.md) | `--resume`/`-r` resumes a specific prior session by UUID | Continuation | 🎯 | 85% | FLAG-VFY | E35, E36 |
| [B20](020_b20_session_id_flag.md) | `--session-id <uuid>` assigns a deterministic UUID to the current session | Session | 🎯 | 80% | FLAG-VFY | E37, E38 |
| [B21](021_b21_fork_session.md) | `--fork-session` creates a new session UUID when resuming; original unchanged | Continuation | 🎯 | 80% | FLAG-VFY | E39, E40 |
| [B22](022_b22_no_session_persistence.md) | `--no-session-persistence` disables session disk writes; only works with `--print` mode | Storage | 🎯 | 85% | FLAG-VFY | E41, E42 |
| [B23](023_b23_session_dir_override.md) | `CLAUDE_CODE_SESSION_DIR` env var overrides session storage directory | Storage | 🎯 | 80% | NEG-ONLY | E43, E44 |
| [B24](024_b24_from_pr.md) | `--from-pr [value]` resumes a session previously linked to a GitHub pull request | Continuation | 🎯 | 75% | FLAG-VFY | E45, E46 |

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
| E43 | B23 | Doc | `../params/057_session_dir.md` | Description | Documents `CLAUDE_CODE_SESSION_DIR` env var that overrides session storage directory |
| E44 | B23 | Test | `../../tests/behavior/b23_session_dir_override.rs` | `b23_session_dir_env_var_not_rejected` | Binary does not explicitly reject `CLAUDE_CODE_SESSION_DIR` env var at startup |
| E45 | B24 | Observation | `claude --help` live output | `--from-pr` flag entry | Help text documents `--from-pr [value]` flag for resuming sessions linked to GitHub pull requests |
| E46 | B24 | Test | `../../tests/behavior/b24_from_pr_flag.rs` | `b24_from_pr_flag_documented_in_help` | `claude --help` output contains `--from-pr` flag |
| E47 | B1, B2 | Test | `../../tests/behavior/b02_new_session.rs` | `b2_continue_flag_proves_separate_sessions` | `--continue` flag exists in `claude --help` — binary-level proof that new-session is the default; presence of a dedicated resume flag implies sessions are separate by default |

---

### Statistical Summary

| Status | Count | IDs |
|--------|-------|-----|
| ✅ Confirmed | 11 | B1, B2, B3, B6, B7, B9, B10, B12, B13, B14, B16 |
| 🎯 Observed | 12 | B4, B5, B8, B11, B15, B18, B19, B20, B21, B22, B23, B24 |
| ⚠️ Exception noted | 1 | B17 (self-contained except at context-compaction boundaries; < 0.2% violation rate) |
| ❓ Uncertain | 1 | B16h |

**Total behaviors:** 25 (B1–B24 + B16h sub-hypothesis; B16h shares B16's row index)
**Confirmed (≥90% certainty):** 11
**Lowest certainty:** B5 (60% — current session selection mechanism)
**Investigation priority:** B5 — can be confirmed by reading Claude Code changelog or source

| Test Tier | Count | IDs |
|-----------|-------|-----|
| VALIDATED | 12 | B1, B2, B6, B7, B9, B10, B12, B13, B14, B15, B17, B18 |
| VALIDATED† | 1 | B5 (distinct mtimes proven; mtime-as-selection-key unproven) |
| FLAG-VFY | 8 | B3, B4, B16, B19, B20, B21, B22, B24 |
| NEG-ONLY | 2 | B11, B23 |
| UNVERIFIED | 1 | B8 |
| MEASURE | 1 | B16h (lim_it; runs by default in container) |

**Validation gap:** 12 of 25 behaviors are fully validated with behavioral assertions.
See `../../-plan/001_behavior_validation_upgrade.plan.md` for the upgrade roadmap.

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
| `b16h_tools_system_prompt.rs` | B16h | MEASURE (lim_it; runs by default in container) |

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
- `../../-plan/001_behavior_validation_upgrade.plan.md` — validation upgrade roadmap
- `../../../module/claude_runner_core/docs/claude_params/` — flag behavior cross-references
