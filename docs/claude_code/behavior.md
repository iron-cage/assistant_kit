# Claude Code Session Behavior

Observed and confirmed behaviors of the `claude` binary relevant to session management, storage,
and flag semantics. Intended for reasoning about session multiplicity and designing
continuation-aware features across the workspace.

See [storage_organization.md](storage_organization.md) for storage layout.
See [jsonl_format.md](jsonl_format.md) for entry-level format details.

---

## Behavior Table

Adapted from hypothesis table format. Status reflects certainty of the observation, not
investigation state. Certainty is based on source evidence — code inspection, observed output,
or direct inference. All behaviors describe the external `claude` binary.

**Status legend:**
- ✅ Confirmed — source code or reproducible test confirms
- 🎯 Observed — seen in practice, mechanism inferred
- ❓ Uncertain — reasonable inference, unconfirmed

| ID  | Behavior | Category | Status | Certainty | Evidence |
|-----|----------|----------|--------|-----------|----------|
| B1  | Default invocation (`claude` with no `--new-session`) continues the most recent session | Continuation | ✅ | 95% | E1, E2, E11 |
| B2  | `--new-session` creates a new `.jsonl` session file; does not append to existing one | Continuation | ✅ | 95% | E1, E12 |
| B3  | `-p` / `--print` controls output mode only (non-interactive capture); does not affect which session is used | Flags | ✅ | 95% | E3, E13 |
| B4  | `-c` / `--continue` is an explicit alias for the default continuation behavior | Flags | 🎯 | 85% | E2, E14 |
| B5  | The "current" session resumed by `--continue` is the most recently modified `.jsonl` file (mtime) | Selection | 🎯 | 60% | E4, E15 |
| B6  | Each project directory accumulates one `.jsonl` file per independent `--new-session` invocation | Storage | ✅ | 90% | E5, E16 |
| B7  | Agent sessions are stored as `agent-*.jsonl` files with `isSidechain: true` in entries; they are siblings, not children | Storage | ✅ | 95% | E6, E17 |
| B8  | Claude Code creates zero-byte `.jsonl` files as session placeholders on startup; they remain if the process crashes before writing entries | Storage | ✅ | 90% | E7, E18 |
| B9  | Claude Code stores project sessions at `~/.claude/projects/{path-encoded}/`; path encoding maps `/` → `-` | Storage | ✅ | 95% | E8, E19 |
| B10 | Claude Code writes entries with `parentUuid` linking each to its predecessor; the root entry has `parentUuid: null` | Entries | ✅ | 95% | E9, E20 |
| B11 | `CLAUDE_CODE_AUTO_CONTINUE` environment variable enables automated continuation mode | Flags | 🎯 | 85% | E10, E21 |
| B12 | Agent JSONL entries carry `sessionId` equal to the parent session UUID (not the agent's own ID) | Families | ✅ | 95% | E22, E26 |
| B13 | New-format agents stored at `{parent-uuid}/subagents/agent-{agentId}.jsonl`; filesystem hierarchy encodes the parent link | Families | ✅ | 95% | E23, E27 |
| B14 | Agent `.meta.json` sidecars contain `agentType` (Explore / general-purpose / Plan) and optional `description` | Families | ✅ | 90% | E24, E28 |
| B15 | Agent entries carry a `slug` field (human-readable label shared by all agents of one parent); root session entries typically lack `slug` | Families | 🎯 | 85% | E25, E29 |
| B16 | `--tools ""` disables all tool invocation; `--tools "default"` restores all tools; both values accepted at CLI parse time | Flags | ✅ | 90% | E30, E31 |
| B16h | Tool *definitions* (~12k tokens) remain in the assembled system prompt even when `--tools ""` is given — invocation is blocked but the token cost is unchanged | Flags | ❓ | 60% | E32 |

---

## Evidence Table

| ID  | Supports | Type        | Source | Location | Content |
|-----|----------|-------------|--------|----------|---------|
| E1  | B1, B2   | Code        | `../../module/claude_runner/src/main.rs` | line 85 | `--new-session  Start a new session (default: continues previous)` — help text confirms continuation as default |
| E2  | B1, B4   | Code        | `../../module/claude_runner_core/src/command.rs` | line 600 | `if self.continue_conversation { parts.push("-c") }` — `-c` is a builder option wrapping the native flag |
| E3  | B3       | Code        | `../../module/claude_runner/src/main.rs` | lines 83, 124 | `-p, --print  Non-interactive mode` and `-p` branch sets print-only; no session flag change |
| E4  | B5       | Inference   | Storage observation | `~/.claude/projects/*/` | Multiple `.jsonl` files in one project; `--continue` must pick one; mtime is the only per-file ordering signal available without metadata |
| E5  | B6       | Observation | Live storage | `~/.claude/projects/…/-commit/` | 25 `.jsonl` files observed in one project directory from repeated sessions |
| E6  | B7       | Observation | Live storage | `~/.claude/projects/*/agent-*.jsonl` | Agent session files observed as siblings of main sessions; entries contain `"isSidechain":true` |
| E7  | B8       | Observation | Live storage | `~/.claude/projects/*/` | Zero-byte `.jsonl` files observed in project directories alongside non-empty sessions |
| E8  | B9       | Observation | Live storage | `~/.claude/projects/` | Project directory names match `/`→`-` encoding of working directory paths |
| E9  | B10      | Doc         | `jsonl_format.md` | `## Conversation Threading` | `parentUuid` links each entry to its parent; null on first entry of a thread |
| E10 | B11      | Code        | `../../module/claude_runner_core/src/command.rs` | line 647-648 | `cmd.env("CLAUDE_CODE_AUTO_CONTINUE", auto_continue.to_string())` — env var set before spawning `claude` |
| E11 | B1       | Test        | `../../module/claude_storage/tests/behavior/b01_default_continues.rs` | `b1_resumable_session_exists` | At least one non-empty non-agent session exists in real `~/.claude/` storage — prerequisite for default continuation |
| E12 | B2       | Test        | `../../module/claude_storage/tests/behavior/b02_new_session.rs` | `b2_new_session_flag_documented` | `claude --help` documents `--new-session` flag |
| E13 | B3       | Test        | `../../module/claude_storage/tests/behavior/b03_print_flag.rs` | `b3_print_flag_documented` | `claude --help` documents `-p` / `--print` as output mode |
| E14 | B4       | Test        | `../../module/claude_storage/tests/behavior/b04_continue_flag.rs` | `b4_continue_flag_documented` | `claude --help` documents `-c` / `--continue` flag |
| E15 | B5       | Test        | `../../module/claude_storage/tests/behavior/b05_mtime_selection.rs` | `b5_sessions_have_distinct_mtimes` | Real project with 2+ sessions has distinct mtimes — mtime ordering is possible |
| E16 | B6       | Test        | `../../module/claude_storage/tests/behavior/b06_session_accumulation.rs` | `b6_project_has_multiple_jsonl` | Real project directory contains 2+ `.jsonl` files |
| E17 | B7       | Test        | `../../module/claude_storage/tests/behavior/b07_agent_sessions.rs` | `b7_agent_session_has_issidechain` | Real `agent-*.jsonl` file contains `"isSidechain":true` in first entry |
| E18 | B8       | Test        | `../../module/claude_storage/tests/behavior/b08_zero_byte_init.rs` | `b8_zero_byte_jsonl_exists` | Zero-byte `.jsonl` files observed in real `~/.claude/` storage |
| E19 | B9       | Test        | `../../module/claude_storage/tests/behavior/b09_storage_path.rs` | `b9_project_dir_names_follow_encoding` | Real project directory names start with `-` (encoded leading `/`) and decode to existing paths |
| E20 | B10      | Test        | `../../module/claude_storage/tests/behavior/b10_entry_threading.rs` | `b10_first_entry_null_parent` | First conversation entry has `parentUuid:null`; second has non-null `parentUuid` |
| E21 | B11      | Test        | `../../module/claude_storage/tests/behavior/b11_auto_continue.rs` | `b11_auto_continue_env_var_recognized` | `claude --version` succeeds with `CLAUDE_CODE_AUTO_CONTINUE=true` set |
| E22 | B12      | Observation | Live storage | `~/.claude/projects/*/subagents/agent-*.jsonl` | Agent entry `sessionId` field equals the parent directory UUID, not the agent filename ID |
| E23 | B13      | Observation | Live storage | `~/.claude/projects/*/` | `{uuid}/subagents/agent-*.jsonl` directories observed; parent UUID in directory name matches root `{uuid}.jsonl` |
| E24 | B14      | Observation | Live storage | `~/.claude/projects/*/subagents/*.meta.json` | `meta.json` files contain `{"agentType":"Explore"}` or `{"agentType":"general-purpose"}` or `{"agentType":"Plan"}`; some include `description` |
| E25 | B15      | Observation | Live storage | `~/.claude/projects/*/subagents/agent-*.jsonl` | All sibling agent entries share identical `slug` value (e.g., `"jaunty-painting-hinton"`); root session first entry has no `slug` (type `queue-operation`) |
| E26 | B12      | Test        | `../../module/claude_storage/tests/behavior/b12_agent_session_id_is_parent.rs` | `b12_agent_session_id_matches_parent_dir` | Agent entry `sessionId` equals the UUID from the parent directory path |
| E27 | B13      | Test        | `../../module/claude_storage/tests/behavior/b13_subagent_directory_structure.rs` | `b13_subagent_dir_exists_for_root_session` | At least one root session has a matching `{uuid}/subagents/` directory |
| E28 | B14      | Test        | `../../module/claude_storage/tests/behavior/b14_agent_meta_json.rs` | `b14_meta_json_contains_agent_type` | Real `.meta.json` file contains `agentType` field with known value |
| E29 | B15      | Test        | `../../module/claude_storage/tests/behavior/b15_agent_slug_field.rs` | `b15_sibling_agents_share_slug` | All sibling agents under one parent share the same `slug` value |
| E30 | B16      | Observation | `claude --help` live output | `--tools` flag entry | Help text: "Specify the list of available tools from the built-in set. Use `""` to disable all tools, `"default"` to use all tools, or specify tool names (e.g. `"Bash,Edit,Read"`)" |
| E31 | B16      | Test        | `../../module/claude_storage/tests/behavior/b16_tools_disable.rs` | `b16a_tools_flag_documented_in_help`, `b16b_tools_empty_string_accepted`, `b16c_tools_default_value_accepted` | Flag documented in help and accepted at CLI parse time without parse error |
| E32 | B16h     | Inference   | Research: Piebald-AI/claude-code-system-prompts; ClaudeLog (2026-04) | Tool assembly layer analysis | Tool definitions injected into assembled system prompt before behavioral flags are applied (confirmed for `--system-prompt` replacement). `--tools` likely operates at invocation-policy layer, not definition-assembly layer — same architectural split as `--system-prompt`. Unconfirmed: requires live token-count comparison. |

---

## Behavior Details

### B1 / B2 — Default continues; `--new-session` creates fresh

Claude Code treats continuation as the default and requires an explicit opt-out:

```
claude                   # continues most recent session
claude --new-session     # starts a new .jsonl file
claude "message"         # continues + sends message (non-interactive)
claude -p "message"      # same as above (explicit --print flag)
```

Each `--new-session` invocation creates exactly one new `.jsonl` file in the project's storage
directory. Over time this produces a directory with one file per distinct session.

### B3 — `-p` is orthogonal to session selection

`-p` / `--print` switches output capture mode. It does not interact with `--new-session` or
`--continue`. A `-p` invocation continues the most recent session unless `--new-session` is
also passed.

### B5 — Current session selection (uncertain)

No explicit "current session pointer" metadata was found in the storage format. The most
probable mechanism is filesystem mtime: `claude` reads the directory listing, sorts by
modification time, and resumes the newest non-agent, non-empty `.jsonl` file.

Certainty is capped at 60% because the Claude Code binary is closed-source and this mechanism
has not been confirmed by source inspection or official documentation.

### B7 — Agent sessions are siblings, not children

Agent sessions (`agent-*.jsonl`) live in the **same directory** as main sessions, not in a
subdirectory. They are distinguished by:
- Filename prefix `agent-`
- `isSidechain: true` in every entry
- `agentId` field present in entries

From the user's perspective they are invisible — `--continue` skips them entirely.

**Note:** B7 describes the flat (old) agent layout. Newer projects use hierarchical layout (B13):
`{uuid}/subagents/agent-{agentId}.jsonl`. Both formats coexist — see B13 for details.

### B8 — Zero-byte session files

Claude Code creates an empty `.jsonl` file as a session placeholder at startup. If the process
crashes before writing any entries, the file remains at 0 bytes.

### B9 — Storage path encoding

Claude Code stores project sessions under `~/.claude/projects/{encoded}/` where the encoded
name is the working directory path with every `/` replaced by `-`. For example,
`/home/user/project` becomes `-home-user-project`.

### B10 — Entry threading

Each JSONL entry contains a `parentUuid` field that links it to the previous entry in the
conversation. The first entry has `"parentUuid": null`. This forms a singly-linked chain
that can be walked to reconstruct conversation order.

### B12 — Agent `sessionId` is the parent UUID

In agent JSONL entries, the `sessionId` field does **not** refer to the agent's own session.
Instead it contains the UUID of the parent (root) session. This is the primary programmatic
link between a sub-agent and the conversation that spawned it.

For example, an agent stored at `43860c56-…/subagents/agent-a6061d6e….jsonl` has
`"sessionId": "43860c56-f828-44bd-953a-432920676b63"` — the parent directory UUID.

### B13 — Subagent directory hierarchy

New-format agent sessions are stored in a subdirectory tree rooted at the parent session UUID:

```
project-dir/
  {parent-uuid}.jsonl                   # root session file
  {parent-uuid}/
    subagents/
      agent-{agentId}.jsonl             # child agent session
      agent-{agentId}.meta.json         # agent metadata sidecar
    tool-results/                       # tool output artifacts
```

The filesystem path itself encodes the parent-child relationship. This supersedes the older
flat layout (B7) where agents were siblings of main sessions in the project root.

Both formats may coexist in real storage (older projects use flat, newer use hierarchical).

### B14 — Agent `meta.json` sidecars

Each agent JSONL file may have a sibling `.meta.json` file containing agent metadata:

```json
{"agentType":"Explore"}
{"agentType":"general-purpose"}
{"agentType":"Plan"}
{"agentType":"claude-code-guide"}
{"agentType":"Explore","description":"Read organizational principles rulebook"}
```

Known `agentType` values (observed distribution): Explore (~63%), general-purpose (~36%),
Plan (<1%), claude-code-guide (rare). The `description` field is optional and present only on some Explore agents.

### B15 — Agent `slug` field

Agent entries carry a `slug` field — a human-readable conversation label like
`"jaunty-painting-hinton"`. All agents spawned from the same parent share an identical slug.

Root session entries typically lack the `slug` field; their first entry is usually of type
`queue-operation` (metadata, not conversation content).

The slug serves as a human-friendly family identifier that could be displayed instead of UUIDs.

---

## Statistical Summary

| Status | Count | IDs |
|--------|-------|-----|
| ✅ Confirmed | 11 | B1, B2, B3, B6, B7, B8, B9, B10, B12, B13, B14 |
| 🎯 Observed | 4 | B4, B5, B11, B15 |
| ❓ Uncertain | 0 | — |

**Total behaviors:** 15
**Confirmed (≥90% certainty):** 11
**Lowest certainty:** B5 (60% — current session selection mechanism)
**Investigation priority:** B5 — can be confirmed by reading Claude Code changelog or source

---

## Invalidation Tests

Each behavior hypothesis has a corresponding invalidation test in
`module/claude_storage/tests/behavior/`. Tests inspect real `~/.claude/` storage to verify
Claude Code's actual output. If Claude Code changes behavior, the tests go RED.

| File | Behavior |
|------|----------|
| `b01_default_continues.rs` | B1 |
| `b02_new_session.rs` | B2 |
| `b03_print_flag.rs` | B3 |
| `b04_continue_flag.rs` | B4 |
| `b05_mtime_selection.rs` | B5 |
| `b06_session_accumulation.rs` | B6 |
| `b07_agent_sessions.rs` | B7 |
| `b08_zero_byte_init.rs` | B8 |
| `b09_storage_path.rs` | B9 |
| `b10_entry_threading.rs` | B10 |
| `b11_auto_continue.rs` | B11 |
| `b12_agent_session_id_is_parent.rs` | B12 |
| `b13_subagent_directory_structure.rs` | B13 |
| `b14_agent_meta_json.rs` | B14 |
| `b15_agent_slug_field.rs` | B15 |

To run:
```bash
cargo nextest run --test behavior
```
