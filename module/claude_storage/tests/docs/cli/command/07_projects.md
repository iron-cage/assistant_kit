# Command :: `.projects`

Integration tests for the `.projects` command. Tests verify summary mode output (default), scope semantics, path anchoring, filter behavior, and exit code contracts.

**Source:** [commands.md#command--7-projects](../../../../docs/cli/commands.md#command--7-projects)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Default (no args) shows active-project summary | Summary Mode (default) |
| IT-2 | scope::relevant includes ancestor project sessions | Scope Behavior |
| IT-3 | scope::under includes descendant project sessions | Scope Behavior |
| IT-4 | scope::global returns all sessions regardless of path | Scope Behavior |
| IT-5 | path:: overrides cwd as scope anchor | Path Anchoring |
| IT-6 | session:: filter narrows results | Filter Behavior |
| IT-7 | min_entries:: filter excludes short sessions | Filter Behavior |
| IT-8 | No matching sessions exits with code 0 | Exit Codes |
| IT-9 | scope::local finds project when path contains underscores | Underscore Path (issue-024) |
| IT-10 | scope::under finds subtree when base path has underscores | Underscore Path (issue-024) |
| IT-11 | scope::relevant finds ancestor when path has underscores | Underscore Path (issue-024) |
| IT-12 | scope::relevant finds topic-scoped ancestor with underscores | Underscore Path (issue-024) |
| IT-13 | scope::under with multiple underscore components finds nested projects | Underscore Path (issue-024) |
| IT-14 | v1 output groups sessions under project path headers | Output Format (plan-004) |
| IT-15 | path header always present at v1 for scope::local single project | Output Format (plan-004) |
| IT-16 | agent sessions collapsed to count line at v1 without agent:: filter | Output Format (plan-004) |
| IT-17 | agent sessions shown individually at v2+ | Output Format (plan-004) |
| IT-18 | entry count shown per session at v2+ | Output Format (plan-004) |
| IT-19 | agent::1 explicit filter disables collapse at v1 | Output Format (plan-004) |
| IT-20 | scope::under displays underscore dirs without splitting at `/` | Underscore Display (issue-029) |
| IT-21 | scope::global displays hyphen-prefixed topic dir in path header | Topic Dir Display (issue-030) |
| IT-22 | scope::under excludes sibling with underscore-suffix name | Sibling Exclusion (issue-031) |
| IT-23 | scope::relevant excludes sibling with underscore-suffix name | Sibling Exclusion (issue-032) |
| IT-24 | entry count shown per session at v1 | Output Format (v1 enhancement) |
| IT-25 | limit::N truncates main sessions shown at v1 | Output Format (v1 enhancement) |
| IT-26 | zero-byte sessions excluded from v1 display | Output Format (v1 enhancement) |
| IT-27 | Summary header format (id, age, count, path) | Summary Mode |
| IT-28 | Truncation gate — message ≤ 50 chars shown in full | Summary Mode |
| IT-29 | Truncation formula — message > 50 chars as first30...last30 | Summary Mode |
| IT-30 | No sessions in scope shows "No active project found." | Summary Mode |
| IT-31 | Explicit scope::local keeps list mode | Filter Passthrough |
| IT-32 | Explicit limit::N keeps list mode | Filter Passthrough |
| IT-33 | Family header format (conversations + agents) | Family Display |
| IT-34 | Per-root agent breakdown [N agents: type summary] | Family Display |
| IT-35 | Hierarchical format detection (subagents/ path) | Family Display |
| IT-36 | Flat format detection (sessionId linkage) | Family Display |
| IT-37 | Orphan family display (root missing) | Family Display |
| IT-38 | Childless root (no bracket suffix) | Family Display |
| IT-39 | Meta.json agentType in breakdown | Family Display |
| IT-40 | Empty/malformed meta.json fallback to "unknown" | Family Display |
| IT-41 | v1 orphan shows `? (orphan)` label (bug-cc-c1) | Family Display |
| IT-42 | v2 root entry count singular `(1 entry)` | Family Display |
| IT-43 | v2 agent entry count singular `1 entry` | Family Display |
| IT-41 | verbosity::1 alone stays in summary mode (bug-is-default-verbosity) | Summary Mode |
| IT-42 | Summary mode shows "Active project" header (task-016) | Project-Centric Output |
| IT-43 | Summary mode shows session count aggregate (task-016) | Project-Centric Output |
| IT-44 | List mode shows projects sorted by recency (task-016) | Project-Centric Output |
| IT-45 | verbosity::0 outputs project paths only (task-016) | Project-Centric Output |
| IT-46 | Topic path shown even when topic dir absent from disk | Topic Existence Guard (issue-035) |
| IT-47 | Topic path shown when topic dir present on disk | Topic Existence Guard (issue-035) |
| IT-48 | Default-topic path shown when topic dir absent from disk | Topic Existence Guard (issue-035) |
| IT-49 | Base path shown correctly with no topic suffix | Topic Existence Guard (issue-035) |
| IT-50 | Double-topic key shows both topic components unconditionally | Topic Existence Guard (issue-035) |

## Test Coverage Summary

- Summary Mode (default): 1 test (IT-1)
- Summary Mode: 5 tests (IT-27–IT-30, IT-41)
- Filter Passthrough: 2 tests (IT-31–IT-32)
- Scope Behavior: 3 tests (IT-2, IT-3, IT-4)
- Path Anchoring: 1 test (IT-5)
- Filter Behavior: 2 tests (IT-6, IT-7)
- Exit Codes: 1 test (IT-8)
- Underscore Path (issue-024): 5 tests (IT-9 through IT-13)
- Output Format (plan-004): 6 tests (IT-14 through IT-19)
- Underscore Display (issue-029): 1 test (IT-20)
- Topic Dir Display (issue-030): 1 test (IT-21)
- Sibling Exclusion (issue-031): 1 test (IT-22)
- Sibling Exclusion (issue-032): 1 test (IT-23)
- Output Format (v1 enhancement): 3 tests (IT-24, IT-25, IT-26)
- Family Display: 11 tests (IT-33 through IT-40, IT-41 through IT-43)
- Project-Centric Output (task-016): 4 tests (IT-42 through IT-45)
- Topic Existence Guard (issue-035): 5 tests (IT-46 through IT-50)

## Test Cases

---

### IT-1: Default (no args) shows active-project summary

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at `/home/user1/pro/alpha` containing at least one session with entries. Run from `/home/user1/pro/alpha`.
- **When:** `clg .projects`
- **Then:** ```
Active project  ~/pro/alpha  (N sessions, last active Xd ago)
Last session:  {8-char-id}  Xd ago  (N entries)

Last message:
  {message text or truncated form}
```
stdout does NOT contain `Found N projects:` (list-mode header absent).; summary header present + `Found N projects:` absent
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-2: scope::relevant includes ancestor project sessions

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b/c`, `/a/b`, and `/a`. Run from `/a/b/c`.
- **When:** `clg .projects scope::relevant`
- **Then:** stdout lists sessions from all three projects: `/a/b/c`, `/a/b`, and `/a`.; + sessions from all ancestor-chain projects present
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-3: scope::under includes descendant project sessions

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b`, `/a/b/c`, `/a/b/c/d`, and `/z` (unrelated). Run from `/a/b`.
- **When:** `clg .projects scope::under`
- **Then:** stdout lists sessions from `/a/b`, `/a/b/c`, and `/a/b/c/d`; not from `/z`.; + sessions from all descendant projects present; unrelated projects absent
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-4: scope::global returns all sessions regardless of path

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b`, `/c/d`, and `/e/f`. Run from `/a/b`.
- **When:** `clg .projects scope::global`
- **Then:** stdout lists sessions from all three projects.; + sessions from all projects in storage
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-5: path:: overrides cwd as scope anchor

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b/c`, `/a/b`, and `/a`. Run from `/tmp` (no project there).
- **When:** `clg .projects scope::local path::/a/b/c`
- **Then:** Sessions from the project at `/a/b/c` only; cwd (`/tmp`) has no effect.; + path parameter used as anchor instead of cwd
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-6: session:: filter narrows results

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing sessions `-commit.jsonl` and `-default_topic.jsonl`. Run from that project.
- **When:** `clg .projects session::commit`
- **Then:** stdout lists only sessions matching "commit" in their ID; `-default_topic` session is absent.; + only sessions with "commit" in ID appear
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-7: min_entries:: filter excludes short sessions

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing: one session with 3 entries and one session with 15 entries. Run from that project.
- **When:** `clg .projects min_entries::10`
- **Then:** stdout lists only the session with 15 entries; the 3-entry session is absent.; + only sessions meeting the entry count threshold appear
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-8: No matching sessions exits with code 0

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (empty storage — no projects). Run from any directory.
- **When:** `clg .projects scope::global`
- **Then:** stdout is empty or contains a "no sessions found" indication; exit code is 0.; + no error on stderr for empty/no-match storage
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-9: scope::local finds project when path contains underscores

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at `/home/user1/wip_core`. Run from `/home/user1/wip_core`.
- **When:** `clg .projects scope::local`
- **Then:** stdout lists the session from `/home/user1/wip_core`; exit code 0.; + session from underscore-path project appears in output
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-10: scope::under finds subtree when base path has underscores

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/home/user1/wip_core` and `/home/user1/wip_core/child`. Run from `/home/user1/wip_core`.
- **When:** `clg .projects scope::under`
- **Then:** stdout lists sessions from both `/home/user1/wip_core` and `/home/user1/wip_core/child`; exit code 0.; + sessions from all underscore-base subtree projects present
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-11: scope::relevant finds ancestor when path has underscores

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/home/user1/wip_core` (ancestor) and `/home/user1/wip_core/sub/child` (current). Run from `/home/user1/wip_core/sub/child`.
- **When:** `clg .projects scope::relevant`
- **Then:** stdout lists sessions from both projects (current + ancestor with underscores); exit code 0.; + sessions from ancestor with underscore path appear
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-12: scope::relevant finds topic-scoped ancestor with underscores

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at `/home/user1/wip_core` with topic `default_topic` (storage dir ends in `--default-topic`). Run from `/home/user1/wip_core/child`.
- **When:** `clg .projects scope::relevant`
- **Then:** stdout lists sessions from the topic-scoped ancestor project; exit code 0.; + topic-scoped underscore-path ancestor sessions appear
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-13: scope::under with multiple underscore components finds nested projects

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `root/my_project/sub_module` (base), `root/my_project/sub_module/feature_x` (child), and `root/other_project` (unrelated). Run with `path::root/my_project/sub_module`.
- **When:** `clg .projects scope::under path::root/my_project/sub_module`
- **Then:** stdout lists sessions from base and child; sessions from `root/other_project` are absent.; + multi-underscore-component base + child sessions both appear; unrelated session absent
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-14: v1 output groups sessions under project path headers

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with two path-based projects (e.g., `/tmp/proj-a` and `/tmp/proj-b`), one session each.
- **When:** `clg .projects scope::global verbosity::1`
- **Then:** ```
Found 2 sessions:

/tmp/proj-a: (1 session)
  * session-id-a  Xs ago  (2 entries)

/tmp/proj-b: (1 session)
  * session-id-b  Xs ago  (2 entries)
```; + path headers present + sessions grouped below them
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-15: path header always present at v1 for scope::local single project

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with one path project at a known path. Run with `path::` pointing to that project.
- **When:** `clg .projects scope::local path::{project} verbosity::1`
- **Then:** stdout contains a line like `/path/to/project: (1 session)` followed by `  * {session-id}`.; + path header present
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-16: agent sessions collapsed to count line at v1 without agent:: filter

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with one project containing 2 main sessions (`session-main-a`, `session-main-b`) and 3 agent sessions (`agent-task-001`, `agent-task-002`, `agent-task-003`).
- **When:** `clg .projects scope::global verbosity::1`
- **Then:** ```
Found 5 sessions:

/path/to/project: (5 sessions)
  * session-main-a  Xs ago  (2 entries)
  - session-main-b  Xs ago  (2 entries)
  + 3 agent sessions (last: Xs ago)
```; + agents collapsed + main sessions listed individually
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-17: agent sessions shown individually at v2+

- **Given:** Same as IT-16 (2 main + 3 agent sessions in one project).
- **When:** `clg .projects scope::global verbosity::2`
- **Then:** + all 5 sessions listed individually + no collapse line
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-18: entry count shown per session at v2+

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with one project and one session containing exactly 4 entries.
- **When:** `clg .projects scope::global verbosity::2`
- **Then:** ```
Found 1 session:

~/path/to/project:
  - session-id  (4 entries)
```; + `(4 entries)` present in output
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-19: agent::1 explicit filter disables collapse at v1

- **Given:** Same as IT-16 (2 main + 3 agent sessions in one project).
- **When:** `clg .projects scope::global verbosity::1 agent::1`
- **Then:** + agent sessions listed individually when agent::1 set
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-20: scope::under displays underscore dirs without splitting at `/`

- **Given:** Create real filesystem directories `/tmp/{tempdir}/wip_core/myproject/` so the FS-guided decoder can verify the correct path. `export CLAUDE_STORAGE_ROOT` pointing to a fixture root with a session in the path-encoded `wip_core/myproject` project.
- **When:** `clg .projects scope::under path::/tmp/{tempdir}/wip_core verbosity::1`
- **Then:** stdout contains a line with `wip_core` in the project path header; no line contains `wip/core`.; + `wip_core` present in header + `wip/core` absent
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-21: scope::global displays hyphen-prefixed topic dir in path header

- **Given:** Create real filesystem directory `{tempdir}/src/-default_topic/`. Write a session for the project at that path. `export CLAUDE_STORAGE_ROOT` and `HOME` to the temp dir.
- **When:** `clg .projects scope::global verbosity::1`
- **Then:** stdout path header contains `-default_topic`; no line ends with `src:` (truncated form absent).; + `-default_topic` in header + `src:` truncation absent
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-22: scope::under excludes sibling with underscore-suffix name

- **Given:** Create real filesystem directories `{tempdir}/base/sub/` (child) and `{tempdir}/base_extra/` (sibling). Write session `session-it25-child` for the child and `session-it25-sibling` for the sibling. `export CLAUDE_STORAGE_ROOT` and `HOME` to the temp dir.
- **When:** `clg .projects scope::under path::{tempdir}/base`
- **Then:** stdout contains `session-it25-child`; stdout does NOT contain `session-it25-sibling`.; + child session present + sibling session absent
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-23: scope::relevant excludes sibling with underscore-suffix name

- **Given:** Create real filesystem directories `{tempdir}/base/` (sibling) and `{tempdir}/base_extra/` (cwd). Write session `session-it26-sibling` for `base` and `session-it26-current` for `base_extra`. `export CLAUDE_STORAGE_ROOT` and `HOME` to the temp dir.
- **When:** `clg .projects scope::relevant path::{tempdir}/base_extra`
- **Then:** stdout contains `session-it26-current`; stdout does NOT contain `session-it26-sibling`.; + current session present + sibling session absent
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-24: entry count shown per session at v1

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with one project and one session containing exactly 4 entries.
- **When:** `clg .projects scope::global verbosity::1`
- **Then:** ```
Found 1 session:

/path/to/project: (1 session)
  * session-id  Xs ago  (4 entries)
```; + `(4 entries)` present at v1
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-25: limit::N truncates main sessions shown at v1

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with one project containing 5 main sessions.
- **When:** `clg .projects scope::global verbosity::1 limit::2`
- **Then:** + truncation hint present with correct count
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-26: zero-byte sessions excluded from v1 display

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with one project containing one real session (`session-real`, 2 entries) and one zero-byte file (`session-placeholder.jsonl`).
- **When:** `clg .projects scope::global verbosity::1`
- **Then:** + real session present + zero-byte placeholder absent
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-27: Summary header format (path, count, age, last-session)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing one session with a known UUID and a known number of entries. Run from the project directory.
- **When:** `clg .projects`
- **Then:** ```
Active project  {path}  (N sessions, last active Xd ago)
Last session:  {8-char-id}  Xd ago  (N entries)
```; + header fields present: project path, session count, `Last session:` line with entry count
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-28: Truncation gate — message ≤ 50 chars shown in full

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing a session whose last text entry is exactly 40 characters (e.g. `Fix typo in the readme file near line 10`). Run from that project.
- **When:** `clg .projects`
- **Then:** The `Last message:` section shows the full 40-char string; no `...` appears in the output.; + full message shown + no ellipsis
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-29: Truncation formula — message > 50 chars as first30...last30

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing a session whose last text entry is exactly 60 characters, with distinct known first-30 and last-30 substrings. Run from that project.
- **When:** `clg .projects`
- **Then:** The `Last message:` section shows `{first30}...{last30}`. The full 60-char source text does NOT appear verbatim.; + `...` present + first30 and last30 substrings match fixture
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-30: No sessions in scope shows "No active project found."

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (empty storage — no session files). Run from any directory.
- **When:** `clg .projects`
- **Then:** `No active project found.`; + `No active project found.` in stdout
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-31: Explicit scope::local keeps list mode

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing at least one session. Run from that project.
- **When:** `clg .projects scope::local`
- **Then:** stdout contains `Found N project` (list-mode header); no `Active project` line.; + `Found N project` header present + `Active project` absent
**⚠️ Maintenance:** The negative check string (`Active project`) must match the current summary-mode header. If the header is renamed, update this test assertion. History: `Active session` (task-007) → `Active project` (task-016)
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-32: Explicit limit::N keeps list mode

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing at least one session. Run from that project.
- **When:** `clg .projects limit::5`
- **Then:** stdout contains `Found N project` (list-mode header); no `Active project` line.; + `Found N project` header present + `Active project` absent
**⚠️ Maintenance:** The negative check string (`Active project`) must match the current summary-mode header. If the header is renamed, update this test assertion. History: `Active session` (task-007) → `Active project` (task-016)
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-33: Family header format (conversations + agents)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project containing 1 root session and 3 agent sessions in hierarchical layout (`{uuid}/subagents/`).
- **When:** `clg .projects scope::local`
- **Then:** Header contains `conversations` and `agents`.; + family header format + no legacy collapse
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-34: Per-root agent breakdown [N agents: type summary]

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project containing 1 root and 3 agents (2×Explore, 1×general-purpose) in hierarchical layout with meta.json sidecars.
- **When:** `clg .projects scope::local`
- **Then:** Root session line contains `[3 agents: 2×Explore, 1×general-purpose]`.; + bracket breakdown present with correct counts and types
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-35: Hierarchical format detection (subagents/ path)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with 2 root sessions, each with distinct agents in their own `{uuid}/subagents/` directory.
- **When:** `clg .projects scope::local`
- **Then:** Each root line shows only its own agent count, not the total.; + agents attributed to correct parent
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-36: Flat format detection (sessionId linkage)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with 1 root session and 2 flat agent files. Each agent's first JSONL entry has `"sessionId"` matching the root UUID.
- **When:** `clg .projects scope::local`
- **Then:** Root line shows `[2 agents:` breakdown.; + flat agents attributed to parent via sessionId
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-37: Orphan family display (root missing)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with `{uuid}/subagents/agent-*.jsonl` but NO `{uuid}.jsonl` root file.
- **When:** `clg .projects scope::local`
- **Then:** Output contains `?` marker on the orphan line.; + orphan marker present
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-38: Childless root (no bracket suffix)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with 1 root session and 0 agents.
- **When:** `clg .projects scope::local`
- **Then:** Root line has mtime and entry count but no `[` character.; + no bracket on childless root
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-39: Meta.json agentType in breakdown

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with 1 root and 1 agent in hierarchical layout. The agent's `meta.json` contains `{"agentType":"Plan"}`.
- **When:** `clg .projects scope::local`
- **Then:** Root line contains `Plan` in the bracket breakdown.; + meta.json agentType shown in breakdown
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-40: Empty/malformed meta.json fallback to "unknown"

- **Given:** 1 root + 1 hierarchical agent, each with 1 JSONL entry.
- **When:** `clg .projects scope::local verbosity::2`
- **Then:** stdout contains `1 entry` and does NOT contain `1 entries`; + correct singular noun for agent entry count
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-41: verbosity::1 alone stays in summary mode (bug-is-default-verbosity)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing at least one session with entries. Run from that project.
- **When:** `clg .projects verbosity::1`
- **Then:** Same summary block as bare `clg .projects` — NOT a project list.
```
Active project  ~/path/to/project  (N sessions, last active Xd ago)
Last session:  {8-char-id}  Xd ago  (N entries)

Last message:
  {message text}
```
stdout does NOT contain `Found N projects:` (list-mode header must be absent).; + summary header present + `Found N projects:` absent

**Root Cause (bug-is-default-verbosity):** `is_default` gate in `projects_routine` included `verbosity` in its all-None check (`cmd.get_integer("verbosity").is_none()`). Passing `verbosity::1` returned `Some(1)` instead of `None`, setting `is_default=false` and routing to list mode even though `verbosity::1` is semantically equivalent to the default
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-42: Summary mode shows "Active project" header (task-016)

- **Given:** One project at cwd with ≥1 session.
- **When:** `clg .projects`
- **Then:** stdout first line starts with `Active project`.; + `Active project` present + `Active session` absent
- **Exit:** 0
- **Source:** `tests/projects_output_format_test.rs::it_summary_mode_shows_active_project_header`

---

### IT-43: Summary mode shows session count aggregate (task-016)

- **Given:** One project at cwd with 3 sessions.
- **When:** `clg .projects`
- **Then:** stdout contains `sessions,`.; + session count aggregate present
- **Exit:** 0
- **Source:** `tests/projects_output_format_test.rs::it_summary_mode_shows_session_count`

---

### IT-44: List mode shows projects sorted by recency (task-016)

- **Given:** Two projects (`proj_alpha` and `proj_beta`) with different file mtimes. `proj_beta` has a newer mtime.
- **When:** `clg .projects scope::global`
- **Then:** `proj_beta` appears before `proj_alpha` in stdout.; + recency-first ordering confirmed
- **Exit:** 0
- **Source:** `tests/projects_output_format_test.rs::it_list_mode_shows_projects_sorted_by_recency`

---

### IT-45: verbosity::0 outputs project paths only (task-016)

- **Given:** One project with ≥1 session.
- **When:** `clg .projects scope::global verbosity::0`
- **Then:** One line containing the project path; no other output.; + project path present + no `sessions,` or `Found` text
- **Exit:** 0
- **Source:** `tests/projects_output_format_test.rs::it_verbosity_0_shows_paths_only`

---

### IT-46: Topic path shown even when topic dir absent from disk

- **Given:** Storage root with one project dir `{encoded}--commit` containing one session. The `-commit` filesystem directory does NOT exist under the project path.
- **When:** `clg .projects scope::local`
- **Then:** Path header contains `/-commit` (topic component appended regardless of disk state).; + `/-commit` present in path header
- **Exit:** 0
- **Source:** `tests/projects_path_encoding_test.rs::projects_shows_topic_path_when_topic_dir_absent`

---

### IT-47: Topic path shown when topic dir present on disk

- **Given:** Storage root with one project dir `{encoded}--commit` containing one session. The `-commit` filesystem directory DOES exist.
- **When:** `clg .projects scope::local`
- **Then:** Path header contains `/-commit`.; + `/-commit` present in path header
- **Exit:** 0
- **Source:** `tests/projects_path_encoding_test.rs::projects_shows_topic_path_when_topic_dir_present`

---

### IT-48: Default-topic path shown when topic dir absent from disk

- **Given:** Storage root with one project dir `{encoded}--default-topic` containing one session. The `-default_topic` filesystem directory does NOT exist.
- **When:** `clg .projects scope::local`
- **Then:** Path header contains `/-default_topic`.; + `/-default_topic` present in path header
- **Exit:** 0
- **Source:** `tests/projects_path_encoding_test.rs::projects_shows_default_topic_path_when_topic_dir_absent`

---

### IT-49: Base path shown correctly with no topic suffix

- **Given:** Storage root with one plain project dir `{encoded}` (no `--` suffix) containing one session.
- **When:** `clg .projects scope::local`
- **Then:** Path header shows the decoded base path without any `/-topic` suffix.; + base path present + no spurious topic suffix
- **Exit:** 0
- **Source:** `tests/projects_path_encoding_test.rs::projects_shows_base_path_with_no_topic`

---

### IT-50: Double-topic key shows both topic components unconditionally

- **Given:** Storage root with one project dir `{encoded_base}--default-topic--commit`. Topic dirs (`-default_topic`, `-commit`) are NOT created on disk.
- **When:** `clg .projects scope::global verbosity::1`
- **Then:** Path header contains `/-default_topic` AND `/-commit`.; + both topic components present in path
- **Exit:** 0
- **Source:** `tests/projects_path_encoding_test.rs::projects_shows_both_topic_components_for_double_topic_key`
