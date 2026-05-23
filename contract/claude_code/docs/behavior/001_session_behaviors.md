# Claude Code: Session Behaviors

### Scope

- **Purpose**: Catalog observed and confirmed external behaviors of the `claude` binary relevant to session lifecycle and storage.
- **Responsibility**: Authoritative source of behavior hypotheses (B1–B24) with evidence and invalidation tests.
- **In Scope**: Session continuation, flag semantics, agent layouts, entry threading, storage path encoding, cross-session relationship absence (conversation chain foundations).
- **Out of Scope**: Entry-level JSONL schema (→ [004_jsonl_format.md](004_jsonl_format.md)); storage directory structure (→ [002_storage_organization.md](002_storage_organization.md)).

---

### Behavior Table

Adapted from hypothesis table format. Status reflects certainty of the observation, not
investigation state. Certainty is based on source evidence — code inspection, observed output,
or direct inference. All behaviors describe the external `claude` binary.

**Status legend:**
- ✅ Confirmed — source code or reproducible test confirms
- 🎯 Observed — seen in practice, mechanism inferred
- ❓ Uncertain — reasonable inference, unconfirmed

| ID  | Behavior | Category | Status | Certainty | Evidence |
|-----|----------|----------|--------|-----------|----------|
| B1  | `claude` binary defaults to starting a NEW session on each invocation; resuming the most recent session requires explicit `--continue`/`-c`. Note: the `clr` wrapper inverts this default by passing `-c` by default | Continuation | ✅ | 90% | E1, E2, E11 |
| B2  | Each `claude` invocation without `--continue` creates a separate new `.jsonl` session file; sessions are not appended to existing ones. Note: `--new-session` is a `clr` wrapper flag (absent from `claude --help`) that suppresses the wrapper's default `-c` to restore this binary-default behavior | Storage | ✅ | 95% | E1, E12 |
| B3  | `-p` / `--print` controls output mode only (non-interactive capture); does not affect which session is used | Flags | ✅ | 95% | E3, E13 |
| B4  | `-c` / `--continue` is the explicit opt-in for resuming the most recently modified session; at the binary level continuation is NOT the default — it must be requested with `-c` | Flags | 🎯 | 85% | E2, E14 |
| B5  | The "current" session resumed by `--continue` is the most recently modified `.jsonl` file (mtime) | Selection | 🎯 | 60% | E4, E15 |
| B6  | Each project directory accumulates one `.jsonl` file per independent session invocation (each call without `--continue`); session files are never compacted or rotated | Storage | ✅ | 90% | E5, E16 |
| B7  | Agent sessions are stored as `agent-*.jsonl` files with `isSidechain: true` in entries; they are siblings, not children | Storage | ✅ | 95% | E6, E17 |
| B8  | Claude Code creates zero-byte `.jsonl` files as session placeholders on startup; they remain if the process crashes before writing entries | Storage | 🎯 | 85% | E7, E18 |
| B9  | Claude Code stores project sessions at `~/.claude/projects/{path-encoded}/`; path encoding maps `/` → `-` | Storage | ✅ | 95% | E8, E19 |
| B10 | Claude Code writes entries with `parentUuid` linking each to its predecessor; the root entry has `parentUuid: null` | Entries | ✅ | 95% | E9, E20 |
| B11 | `CLAUDE_CODE_AUTO_CONTINUE` environment variable enables automated continuation mode | Flags | 🎯 | 85% | E10, E21 |
| B12 | Agent JSONL entries carry `sessionId` equal to the parent session UUID (not the agent's own ID) | Families | ✅ | 95% | E22, E26 |
| B13 | New-format agents stored at `{parent-uuid}/subagents/agent-{agentId}.jsonl`; filesystem hierarchy encodes the parent link | Families | ✅ | 95% | E23, E27 |
| B14 | Agent `.meta.json` sidecars contain `agentType` (Explore / general-purpose / Plan) and optional `description` | Families | ✅ | 90% | E24, E28 |
| B15 | Agent entries carry a `slug` field (human-readable label shared by all agents of one parent); root session entries typically lack `slug` | Families | 🎯 | 85% | E25, E29 |
| B16 | `--tools ""` disables all tool invocation; `--tools "default"` restores all tools; both values accepted at CLI parse time | Flags | ✅ | 90% | E30, E31 |
| B16h | Tool *definitions* (~12k tokens) remain in the assembled system prompt even when `--tools ""` is given — invocation is blocked but the token cost is unchanged | Flags | ❓ | 60% | E32 |
| B17 | The `parentUuid` chain within one session file is self-contained: every UUID referenced by a `parentUuid` field exists as a `uuid` field within the same `.jsonl` file; no `parentUuid` points across session files | Entries | 🎯 | 85% | E33 |
| B18 | Claude Code writes no cross-session continuation metadata: a new session's first entry has `parentUuid: null` with no field referencing the prior session; logical conversation chains must be inferred externally (e.g., from mtime ordering or content) | Continuation | 🎯 | 80% | E34 |
| B19 | `--resume` / `-r` resumes a specific prior session by UUID; appends to that session's `.jsonl` file rather than the most recently modified one | Continuation | 🎯 | 85% | E35, E36 |
| B20 | `--session-id <uuid>` assigns a deterministic UUID to the current session instead of auto-generating one; if the UUID matches an existing file, behavior follows other flags (`--resume`, `--fork-session`) | Session | 🎯 | 80% | E37, E38 |
| B21 | `--fork-session` creates a new session UUID when resuming; the resumed history is copied into a new `.jsonl` file, preserving the original session unchanged | Continuation | 🎯 | 80% | E39, E40 |
| B22 | `--no-session-persistence` disables session disk writes; no `.jsonl` file is created and the session cannot be resumed; only works with `--print` mode | Storage | 🎯 | 85% | E41, E42 |
| B23 | `CLAUDE_CODE_SESSION_DIR` env var overrides the directory where session `.jsonl` files are stored; when set, Claude reads/writes session files from this path instead of `~/.claude/projects/{encoded-path}/` | Storage | 🎯 | 80% | E43, E44 |
| B24 | `--from-pr [value]` resumes a session previously linked to a GitHub pull request; with no argument opens an interactive picker; with PR number/URL resumes the associated session directly | Continuation | 🎯 | 75% | E45, E46 |

---

### Evidence Table

| ID  | Supports | Type        | Source | Location | Content |
|-----|----------|-------------|--------|----------|---------|
| E1  | B1, B2   | Code        | `../../../../module/claude_runner/src/main.rs` | line 85 | `--new-session  Start a new session (default: continues previous)` — `clr` wrapper help text; confirms wrapper default is continuation (not the `claude` binary native default) |
| E2  | B1, B4   | Code        | `../../../../module/claude_runner_core/src/command.rs` | line 600 | `if self.continue_conversation { parts.push("-c") }` — `-c` is a builder option wrapping the native flag |
| E3  | B3       | Code        | `../../../../module/claude_runner/src/main.rs` | lines 83, 124 | `-p, --print  Non-interactive mode` and `-p` branch sets print-only; no session flag change |
| E4  | B5       | Inference   | Storage observation | `~/.claude/projects/*/` | Multiple `.jsonl` files in one project; `--continue` must pick one; mtime is the only per-file ordering signal available without metadata |
| E5  | B6       | Observation | Live storage | `~/.claude/projects/…/-commit/` | 25 `.jsonl` files observed in one project directory from repeated sessions |
| E6  | B7       | Observation | Live storage | `~/.claude/projects/*/agent-*.jsonl` | Agent session files observed as siblings of main sessions; entries contain `"isSidechain":true` |
| E7  | B8       | Observation | Live storage | `~/.claude/projects/*/` | Zero-byte `.jsonl` files observed in project directories alongside non-empty sessions |
| E8  | B9       | Observation | Live storage | `~/.claude/projects/` | Project directory names match `/`→`-` encoding of working directory paths |
| E9  | B10      | Doc         | `jsonl_format.md` | `## Conversation Threading` | `parentUuid` links each entry to its parent; null on first entry of a thread |
| E10 | B11      | Code        | `../../../../module/claude_runner_core/src/command.rs` | line 647-648 | `cmd.env("CLAUDE_CODE_AUTO_CONTINUE", auto_continue.to_string())` — env var set before spawning `claude` |
| E11 | B1       | Test        | `../../tests/behavior/b001_default_continues.rs` | `b1_resumable_session_exists_in_real_storage` | At least one non-empty non-agent session exists in real `~/.claude/` storage — prerequisite for default continuation |
| E12 | B2       | Test        | `../../tests/behavior/b002_new_session.rs` | `b2_multiple_session_files_exist_in_real_project` | At least one project in real `~/.claude/` storage has 2+ non-empty non-agent `.jsonl` files — evidence of per-session file creation |
| E13 | B3       | Test        | `../../tests/behavior/b003_print_flag.rs` | `b3_print_flag_documented_as_output_mode` | `claude --help` documents `-p` / `--print` as output mode |
| E14 | B4       | Test        | `../../tests/behavior/b004_continue_flag.rs` | `b4_continue_flag_documented_in_help` | `claude --help` documents `-c` / `--continue` flag |
| E15 | B5       | Test        | `../../tests/behavior/b005_mtime_selection.rs` | `b5_real_sessions_have_distinct_mtimes` | Real project with 2+ sessions has distinct mtimes — mtime ordering is possible |
| E16 | B6       | Test        | `../../tests/behavior/b006_session_accumulation.rs` | `b6_sessions_accumulate_in_real_project` | Real project directory contains 5+ `.jsonl` files (all types) — higher threshold than B2 (>= 2) to confirm long-term accumulation without rotation |
| E17 | B7       | Test        | `../../tests/behavior/b007_agent_sessions.rs` | `b7_real_agent_session_has_issidechain_true` | Real `agent-*.jsonl` file contains `"isSidechain":true` in first entry |
| E18 | B8       | Observation | `../../tests/behavior/b008_zero_byte_init.rs` | `b8_zero_byte_jsonl_exists_in_real_storage` | Zero-byte `.jsonl` files observed in real `~/.claude/` storage (test logs observation but does not assert — no hard invalidation) |
| E19 | B9       | Test        | `../../tests/behavior/b009_storage_path.rs` | `b9_project_dir_names_follow_encoding_convention` | Real project directory names start with `-` (encoded leading `/`) and decode to existing paths |
| E20 | B10      | Test        | `../../tests/behavior/b010_entry_threading.rs` | `b010_first_entry_has_null_parent_uuid`, `b010_subsequent_entries_have_non_null_parent_uuid` | First conversation entry has `parentUuid:null`; second entry has non-null `parentUuid` referencing first |
| E21 | B11      | Test        | `../../tests/behavior/b011_auto_continue.rs` | `b011_auto_continue_env_var_recognized` | Binary does not print `CLAUDE_CODE_AUTO_CONTINUE` in stderr when env var is set — negative assertion; does not assert on exit code (exit code is trivially 0 and cannot distinguish env var acceptance from ignorance) |
| E22 | B12      | Observation | Live storage | `~/.claude/projects/*/subagents/agent-*.jsonl` | Agent entry `sessionId` field equals the parent directory UUID, not the agent filename ID |
| E23 | B13      | Observation | Live storage | `~/.claude/projects/*/` | `{uuid}/subagents/agent-*.jsonl` directories observed; parent UUID in directory name matches root `{uuid}.jsonl` |
| E24 | B14      | Observation | Live storage | `~/.claude/projects/*/subagents/*.meta.json` | `meta.json` files contain `{"agentType":"Explore"}` or `{"agentType":"general-purpose"}` or `{"agentType":"Plan"}`; some include `description` |
| E25 | B15      | Observation | Live storage | `~/.claude/projects/*/subagents/agent-*.jsonl` | All sibling agent entries share identical `slug` value (e.g., `"jaunty-painting-hinton"`); root session first entry has no `slug` (type `queue-operation`) |
| E26 | B12      | Test        | `../../tests/behavior/b012_agent_session_id_is_parent.rs` | `b012_agent_session_id_matches_parent_dir` | Agent entry `sessionId` equals the UUID from the parent directory path |
| E27 | B13      | Test        | `../../tests/behavior/b013_subagent_directory_structure.rs` | `b013_subagent_dir_exists_for_root_session` | At least one root session has a matching `{uuid}/subagents/` directory |
| E28 | B14      | Test        | `../../tests/behavior/b014_agent_meta_json.rs` | `b014_meta_json_contains_agent_type` | Real `.meta.json` file contains `agentType` field with known value |
| E29 | B15      | Test        | `../../tests/behavior/b015_agent_slug_field.rs` | `b015_sibling_agents_share_slug` | All sibling agents under one parent share the same `slug` value |
| E30 | B16      | Observation | `claude --help` live output | `--tools` flag entry | Help text: "Specify the list of available tools from the built-in set. Use `""` to disable all tools, `"default"` to use all tools, or specify tool names (e.g. `"Bash,Edit,Read"`)" |
| E31 | B16      | Test        | `../../tests/behavior/b016_tools_disable.rs` | `b16a_tools_flag_documented_in_help`, `b16b_tools_empty_string_accepted`, `b16c_tools_default_value_accepted` | Flag documented in help and accepted at CLI parse time without parse error |
| E32 | B16h     | Inference   | Research: Piebald-AI/claude-code-system-prompts; ClaudeLog (2026-04) | Tool assembly layer analysis | Tool definitions injected into assembled system prompt before behavioral flags are applied (confirmed for `--system-prompt` replacement). `--tools` likely operates at invocation-policy layer, not definition-assembly layer — same architectural split as `--system-prompt`. Unconfirmed: requires live token-count comparison. |
| E33 | B17      | Test        | `../../tests/behavior/b017_parentuuid_self_contained.rs` | `it_parentuuid_never_crosses_session_boundary` | Rate-based check: orphaned `parentUuid` references stay below 1% across 10 projects × 5 sessions; zero violations means strict self-containment in sample |
| E34 | B18      | Test        | `../../tests/behavior/b018_no_cross_session_links.rs` | `it_first_entry_parentuuid_is_null` | First conversation entry (user or assistant type) in each session has `parentUuid: null` or absent — no cross-session continuation pointer written |
| E35 | B19      | Observation | `claude --help` live output | `--resume` flag entry | Help text documents `--resume` / `-r <session-id>` flag for resuming a specific prior session by UUID |
| E36 | B19      | Test        | `../../tests/behavior/b019_resume_flag.rs` | `b019_resume_flag_documented_in_help` | `claude --help` output contains `--resume` flag |
| E37 | B20      | Observation | `claude --help` live output | `--session-id` flag entry | Help text documents `--session-id <uuid>` flag for assigning a deterministic UUID to the current session |
| E38 | B20      | Test        | `../../tests/behavior/b020_session_id_flag.rs` | `b020_session_id_flag_documented_in_help` | `claude --help` output contains `--session-id` flag |
| E39 | B21      | Observation | `claude --help` live output | `--fork-session` flag entry | Help text documents `--fork-session` flag for branching from a prior session without modifying the original |
| E40 | B21      | Test        | `../../tests/behavior/b021_fork_session_flag.rs` | `b021_fork_session_flag_documented_in_help` | `claude --help` output contains `--fork-session` flag |
| E41 | B22      | Observation | `claude --help` live output | `--no-session-persistence` flag entry | Help text documents `--no-session-persistence` flag; notes it disables `.jsonl` creation and works only with `--print` mode |
| E42 | B22      | Test        | `../../tests/behavior/b022_no_session_persistence_flag.rs` | `b022_no_session_persistence_flag_documented_in_help` | `claude --help` output contains `--no-session-persistence` flag |
| E43 | B23      | Doc         | `docs/params/057_session_dir.md` | Description | Documents `CLAUDE_CODE_SESSION_DIR` env var that overrides session storage directory to a custom path |
| E44 | B23      | Test        | `../../tests/behavior/b023_session_dir_override.rs` | `b023_session_dir_env_var_not_rejected` | Binary does not explicitly reject `CLAUDE_CODE_SESSION_DIR` env var at startup |
| E45 | B24      | Observation | `claude --help` live output | `--from-pr` flag entry | Help text documents `--from-pr [value]` flag for resuming sessions linked to GitHub pull requests |
| E46 | B24      | Test        | `../../tests/behavior/b024_from_pr_flag.rs` | `b024_from_pr_flag_documented_in_help` | `claude --help` output contains `--from-pr` flag |

---

### Behavior Details

### B1 / B2 — Binary default is new session; wrapper inverts to continuation default

**Binary-level behavior:**

The `claude` binary defaults to a NEW session on every invocation. Continuation requires
an explicit `-c` / `--continue` flag:

```
claude                   # starts a new session (binary default)
claude --continue        # resumes most recently modified session
claude -c "message"      # resumes + sends message (non-interactive)
claude -p "message"      # starts new session; explicit --print flag
```

**Wrapper layer (`clr`) behavior:**

The `clr` wrapper inverts this default by always passing `-c` unless `--new-session` is given.
`--new-session` is a **`clr`-only flag** (absent from `claude --help`); it suppresses the
wrapper's default `-c` to restore binary-default behavior (fresh start):

```
clr                      # passes -c → continues most recent session
clr --new-session        # omits -c → new .jsonl file (binary default)
```

Each session without `--continue` creates exactly one new `.jsonl` file in the project's
storage directory. Over time this produces a directory with one file per distinct session.

### B3 — `-p` is orthogonal to session selection

`-p` / `--print` switches output capture mode. It does not interact with `--continue` or
session creation. A `-p` invocation starts a new session (binary default) unless `-c` /
`--continue` is also passed.

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

### B17 — `parentUuid` chain is self-contained per session file (with known exception)

Within one `.jsonl` session file, the `parentUuid` threading is closed for the vast majority
of entries — no entry references a UUID that lives in a different file.

**Known exception — context-compaction boundaries:** When Claude Code's context window is
exhausted and the conversation is resumed, the continuation user message is appended to the
existing `.jsonl` with a `parentUuid` that references the last UUID from the pre-compaction
context. That UUID may have existed only in the previous context window and was never written
into the file as a top-level `uuid` entry; the orphaned reference is expected and unavoidable.
Empirically, these violations are rare (< 0.2% of entries with a non-null `parentUuid`).

This is the key reason why cross-session conversation chains must be inferred rather than
followed: for B17-conforming entries there is no pointer to jump to, and for the small number
of compaction-boundary exceptions the pointer is dangling. The boundary between two sessions
(even if they represent logically connected work) is a hard storage boundary.

### B18 — No cross-session continuation metadata

When Claude Code starts a new session in a project that already has sessions (whether via a
fresh invocation without `--continue`, or via `--new-session` in the `clr` wrapper), the first
entry of the new session has `parentUuid: null`.
No field in the new session's entries references the prior session's UUID or last entry UUID.

This means:
- Two consecutive sessions in the same project directory look identical from a storage perspective whether they are logically connected or not
- Grouping sessions into Conversations (Conversation Chains) requires heuristic inference — temporal proximity, content context, or external markers
- Claude Code itself has no "resume from prior conversation" semantic in storage; it only "continue current session" (append to the same file) or "start new" (create a new file)

See [007_concept_taxonomy.md](007_concept_taxonomy.md) for how Conversation Chains are defined relative to this storage reality.

### B19 — `--resume` / `-r` resumes by UUID

`--resume <session-id>` (shorthand `-r`) selects a specific `.jsonl` file to resume by UUID
rather than using the most recently modified file. This is the explicit override for B5's
mtime-based selection.

The session UUID must match the filename of an existing `.jsonl` in the project's storage
directory. Combined with `--fork-session`, it branches from that specific checkpoint.

### B20 — `--session-id` assigns deterministic UUID

By default Claude Code generates a random UUIDv4 for each new session. `--session-id <uuid>`
overrides this to a caller-supplied UUID. Useful for reproducible automation where session
identity must be deterministic (e.g., linking Claude invocations to external tracking systems).

If the supplied UUID already matches an existing `.jsonl` file, behavior depends on other flags:
with `--resume` it appends; with `--fork-session` it branches into a new UUID.

### B21 — `--fork-session` branches without modifying original

When resuming a session (`--resume` or `--continue`), `--fork-session` creates a new session
UUID rather than appending to the original file. The resumed history is copied into the new
`.jsonl`, leaving the source session untouched.

This is the mechanism for checkpoint branching: explore alternative conversation paths from a
known-good state without polluting the original session file.

### B22 — `--no-session-persistence` suppresses disk writes

No `.jsonl` file is created. The session exists only for the duration of the invocation and
cannot be resumed. Only works with `--print` mode (non-interactive), since interactive mode
requires session persistence for the terminal UI.

Useful for ephemeral CI queries and privacy-sensitive contexts where session history must not
be written to disk.

### B23 — `CLAUDE_CODE_SESSION_DIR` redirects session storage

When set, session `.jsonl` files are read from and written to the specified path instead of
the default `~/.claude/projects/{encoded-path}/`. Useful for CI pipelines where sessions must
be stored in a known, writable location, or multi-user environments where each user has a
separate session directory.

The env var affects only the current invocation; other Claude Code behavior is unchanged.

### B24 — `--from-pr` resumes a PR-linked session

`--from-pr` without a value opens an interactive picker listing sessions previously associated
with GitHub pull requests. `--from-pr <PR-number-or-URL>` resumes the session linked to that
PR directly.

Requires the original session to have been started with PR context (e.g., via `gh pr checkout`
workflow). This is the inverse of PR-linked session creation: it finds the stored session by PR
identity rather than by UUID or mtime.

---

### Statistical Summary

| Status | Count | IDs |
|--------|-------|-----|
| ✅ Confirmed | 11 | B1, B2, B3, B6, B7, B9, B10, B12, B13, B14, B16 |
| 🎯 Observed | 12 | B4, B5, B8, B11, B15, B18, B19, B20, B21, B22, B23, B24 |
| ⚠️ Exception noted | 1 | B17 (self-contained except at context-compaction boundaries; < 0.2% violation rate) |
| ❓ Uncertain | 1 | B16h |

**Total behaviors:** 24 (B1–B24; B16h is a sub-hypothesis within B16)
**Confirmed (≥90% certainty):** 11
**Lowest certainty:** B5 (60% — current session selection mechanism)
**Investigation priority:** B5 — can be confirmed by reading Claude Code changelog or source

---

### Invalidation Tests

Each behavior hypothesis has a corresponding invalidation test in
`contract/claude_code/tests/behavior/`. Tests inspect real `~/.claude/` storage to verify
Claude Code's actual output. If Claude Code changes behavior, the tests go RED.

| File | Behavior |
|------|----------|
| `b001_default_continues.rs` | B1 |
| `b002_new_session.rs` | B2 |
| `b003_print_flag.rs` | B3 |
| `b004_continue_flag.rs` | B4 |
| `b005_mtime_selection.rs` | B5 |
| `b006_session_accumulation.rs` | B6 |
| `b007_agent_sessions.rs` | B7 |
| `b008_zero_byte_init.rs` | B8 |
| `b009_storage_path.rs` | B9 |
| `b010_entry_threading.rs` | B10 |
| `b011_auto_continue.rs` | B11 |
| `b012_agent_session_id_is_parent.rs` | B12 |
| `b013_subagent_directory_structure.rs` | B13 |
| `b014_agent_meta_json.rs` | B14 |
| `b015_agent_slug_field.rs` | B15 |
| `b016_tools_disable.rs` | B16 (observable: flag accepted; H1 vs H2 requires live API test) |
| `b017_parentuuid_self_contained.rs` | B17 |
| `b018_no_cross_session_links.rs` | B18 |
| `b019_resume_flag.rs` | B19 |
| `b020_session_id_flag.rs` | B20 |
| `b021_fork_session_flag.rs` | B21 |
| `b022_no_session_persistence_flag.rs` | B22 |
| `b023_session_dir_override.rs` | B23 |
| `b024_from_pr_flag.rs` | B24 |
| `b16h_tools_system_prompt.rs` | B16h (live API `lim_it` test; excluded from default filter) |

To run:
```bash
cargo nextest run -p claude_code --test behavior
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [`002_storage_organization.md`](002_storage_organization.md) | Storage directory layout and containment hierarchy |
| doc | [`004_jsonl_format.md`](004_jsonl_format.md) | Entry-level JSONL field schema and content block types |
| doc | [`../params/readme.md`](../params/readme.md) | Canonical definitions for binary flags/env vars (--print, --continue, --resume, --session-id, --fork-session, --no-session-persistence, --from-pr, CLAUDE_CODE_AUTO_CONTINUE, CLAUDE_CODE_SESSION_DIR); --new-session is a `clr` wrapper flag documented in `module/claude_runner/docs/` |
| test | [`../../tests/behavior/`](../../tests/behavior/) | Invalidation test suite — one file per behavior (B1–B24 + B16h) |
| source | [`../../../../module/claude_runner/src/main.rs`](../../../../module/claude_runner/src/main.rs) | Evidence E1–E3: flag definitions (new-session, print, continue) |
| source | [`../../../../module/claude_runner_core/src/command.rs`](../../../../module/claude_runner_core/src/command.rs) | Evidence E2, E10: continuation flag builder and auto-continue env var |
| doc | [`007_concept_taxonomy.md`](007_concept_taxonomy.md) | Conversation Chain and Session hierarchy concepts referenced in B18 |

