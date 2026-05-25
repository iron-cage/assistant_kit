# Command :: `.projects`

Integration tests for the `.projects` command. Tests verify summary mode output (default), scope semantics, path anchoring, filter behavior, and exit code contracts.

**Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | Default (no args) shows active-project summary | Summary Mode (default) |
| INT-2 | scope::relevant includes ancestor project sessions | Scope Behavior |
| INT-3 | scope::under includes descendant project sessions | Scope Behavior |
| INT-4 | scope::global returns all sessions regardless of path | Scope Behavior |
| INT-5 | path:: overrides cwd as scope anchor | Path Anchoring |
| INT-6 | session:: filter narrows results | Filter Behavior |
| INT-7 | min_entries:: filter excludes short sessions | Filter Behavior |
| INT-8 | No matching sessions exits with code 0 | Exit Codes |
| INT-9 | scope::local finds project when path contains underscores | Underscore Path (issue-024) |
| INT-10 | scope::under finds subtree when base path has underscores | Underscore Path (issue-024) |
| INT-11 | scope::relevant finds ancestor when path has underscores | Underscore Path (issue-024) |
| INT-12 | scope::relevant finds topic-scoped ancestor with underscores | Underscore Path (issue-024) |
| INT-13 | scope::under with multiple underscore components finds nested projects | Underscore Path (issue-024) |
| INT-14 | v1 output groups sessions under project path headers | Output Format (plan-004) |
| INT-15 | path header always present at v1 for scope::local single project | Output Format (plan-004) |
| INT-16 | agent sessions collapsed to count line at v1 without agent:: filter | Output Format (plan-004) |
| INT-17 | agent sessions shown individually at v2+ | Output Format (plan-004) |
| INT-18 | entry count shown per session at v2+ | Output Format (plan-004) |
| INT-19 | agent::1 explicit filter disables collapse at v1 | Output Format (plan-004) |
| INT-20 | scope::under displays underscore dirs without splitting at `/` | Underscore Display (issue-029) |
| INT-21 | scope::global displays hyphen-prefixed topic dir in path header | Topic Dir Display (issue-030) |
| INT-22 | scope::under excludes sibling with underscore-suffix name | Sibling Exclusion (issue-031) |
| INT-23 | scope::relevant excludes sibling with underscore-suffix name | Sibling Exclusion (issue-032) |
| INT-24 | entry count shown per session at v1 | Output Format (v1 enhancement) |
| INT-25 | limit::N truncates main sessions shown at v1 | Output Format (v1 enhancement) |
| INT-26 | zero-byte sessions excluded from v1 display | Output Format (v1 enhancement) |
| INT-27 | Summary header format (id, age, count, path) | Summary Mode |
| INT-28 | Truncation gate — message ≤ 50 chars shown in full | Summary Mode |
| INT-29 | Truncation formula — message > 50 chars as first30...last30 | Summary Mode |
| INT-30 | No sessions in scope shows "No active project found." | Summary Mode |
| INT-31 | Explicit scope::local keeps list mode | Filter Passthrough |
| INT-32 | Explicit limit::N keeps list mode | Filter Passthrough |
| INT-33 | Family header format (conversations + agents) | Family Display |
| INT-34 | Per-root agent breakdown [N agents: type summary] | Family Display |
| INT-35 | Hierarchical format detection (subagents/ path) | Family Display |
| INT-36 | Flat format detection (sessionId linkage) | Family Display |
| INT-37 | Orphan family display (root missing) | Family Display |
| INT-38 | Childless root (no bracket suffix) | Family Display |
| INT-39 | Meta.json agentType in breakdown | Family Display |
| INT-40 | Empty/malformed meta.json fallback to "unknown" | Family Display |
| INT-41 | v1 orphan shows `? (orphan)` label (bug-cc-c1) | Family Display |
| INT-42 | v2 root entry count singular `(1 entry)` | Family Display |
| INT-43 | v2 agent entry count singular `1 entry` | Family Display |
| INT-41 | verbosity::1 alone stays in summary mode (bug-is-default-verbosity) | Summary Mode |
| INT-42 | Summary mode shows "Active project" header (task-016) | Project-Centric Output |
| INT-43 | Summary mode shows session count aggregate (task-016) | Project-Centric Output |
| INT-44 | List mode shows projects sorted by recency (task-016) | Project-Centric Output |
| INT-45 | verbosity::0 outputs project paths only (task-016) | Project-Centric Output |
| INT-46 | Topic path shown even when topic dir absent from disk | Topic Existence Guard (issue-035) |
| INT-47 | Topic path shown when topic dir present on disk | Topic Existence Guard (issue-035) |
| INT-48 | Default-topic path shown when topic dir absent from disk | Topic Existence Guard (issue-035) |
| INT-49 | Base path shown correctly with no topic suffix | Topic Existence Guard (issue-035) |
| INT-50 | Double-topic key shows both topic components unconditionally | Topic Existence Guard (issue-035) |

## Test Coverage Summary

- Summary Mode (default): 1 test (INT-1)
- Summary Mode: 5 tests (INT-27–INT-30, INT-41)
- Filter Passthrough: 2 tests (INT-31–INT-32)
- Scope Behavior: 3 tests (INT-2, INT-3, INT-4)
- Path Anchoring: 1 test (INT-5)
- Filter Behavior: 2 tests (INT-6, INT-7)
- Exit Codes: 1 test (INT-8)
- Underscore Path (issue-024): 5 tests (INT-9 through INT-13)
- Output Format (plan-004): 6 tests (INT-14 through INT-19)
- Underscore Display (issue-029): 1 test (INT-20)
- Topic Dir Display (issue-030): 1 test (INT-21)
- Sibling Exclusion (issue-031): 1 test (INT-22)
- Sibling Exclusion (issue-032): 1 test (INT-23)
- Output Format (v1 enhancement): 3 tests (INT-24, INT-25, INT-26)
- Family Display: 11 tests (INT-33 through INT-40, INT-41 through INT-43)
- Project-Centric Output (task-016): 4 tests (INT-42 through INT-45)
- Topic Existence Guard (issue-035): 5 tests (INT-46 through INT-50)

## Test Cases

---

### INT-1: Default (no args) shows active-project summary

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects
```

**Expected behavior:**
- Fixture: project at `/home/alice/projects/alpha` containing at least one session with entries; run from `/home/alice/projects/alpha`
- Output contains:
  ```
  Active project  ~/projects/alpha  (N sessions, last active Xd ago)
  Last session:  {8-char-id}  Xd ago  (N entries)

  Last message:
    {message text or truncated form}
  ```
- stdout does NOT contain `Found N projects:` (list-mode header absent)
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-2: scope::relevant includes ancestor project sessions

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::relevant
```

**Expected behavior:**
- Fixture: projects at `/a/b/c`, `/a/b`, and `/a`; run from `/a/b/c`
- stdout lists sessions from all three projects: `/a/b/c`, `/a/b`, and `/a`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-3: scope::under includes descendant project sessions

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::under
```

**Expected behavior:**
- Fixture: projects at `/a/b`, `/a/b/c`, `/a/b/c/d`, and `/z` (unrelated); run from `/a/b`
- stdout lists sessions from `/a/b`, `/a/b/c`, and `/a/b/c/d`; not from `/z`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-4: scope::global returns all sessions regardless of path

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global
```

**Expected behavior:**
- Fixture: projects at `/a/b`, `/c/d`, and `/e/f`
- stdout lists sessions from all three projects
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-5: path:: overrides cwd as scope anchor

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local path::/a/b/c
```

**Expected behavior:**
- Fixture: projects at `/a/b/c`, `/a/b`, and `/a`; run from `/tmp` (no project there)
- Sessions from the project at `/a/b/c` only; cwd (`/tmp`) has no effect
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-6: session:: filter narrows results

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects session::commit
```

**Expected behavior:**
- Fixture: project at cwd containing sessions `-commit.jsonl` and `-default_topic.jsonl`; run from that project
- stdout lists only sessions matching "commit" in their ID; `-default_topic` session is absent
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-7: min_entries:: filter excludes short sessions

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects min_entries::10
```

**Expected behavior:**
- Fixture: project at cwd containing one session with 3 entries and one session with 15 entries; run from that project
- stdout lists only the session with 15 entries; the 3-entry session is absent
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-8: No matching sessions exits with code 0

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global
```

**Expected behavior:**
- Fixture: empty storage — no projects
- stdout is empty or contains a "no sessions found" indication; no error on stderr
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-9: scope::local finds project when path contains underscores

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local
```

**Expected behavior:**
- Fixture: project at `/home/alice/my_project`; run from `/home/alice/my_project`
- stdout lists the session from `/home/alice/my_project`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-10: scope::under finds subtree when base path has underscores

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::under
```

**Expected behavior:**
- Fixture: projects at `/home/alice/my_project` and `/home/alice/my_project/child`; run from `/home/alice/my_project`
- stdout lists sessions from both `/home/alice/my_project` and `/home/alice/my_project/child`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-11: scope::relevant finds ancestor when path has underscores

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::relevant
```

**Expected behavior:**
- Fixture: projects at `/home/alice/my_project` (ancestor) and `/home/alice/my_project/sub/child` (current); run from `/home/alice/my_project/sub/child`
- stdout lists sessions from both projects (current + ancestor with underscores)
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-12: scope::relevant finds topic-scoped ancestor with underscores

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::relevant
```

**Expected behavior:**
- Fixture: project at `/home/alice/my_project` with topic `default_topic` (storage dir ends in `--default-topic`); run from `/home/alice/my_project/child`
- stdout lists sessions from the topic-scoped ancestor project
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-13: scope::under with multiple underscore components finds nested projects

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::under path::root/my_project/sub_module
```

**Expected behavior:**
- Fixture: projects at `root/my_project/sub_module` (base), `root/my_project/sub_module/feature_x` (child), and `root/other_project` (unrelated)
- stdout lists sessions from base and child; sessions from `root/other_project` are absent
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-14: v1 output groups sessions under project path headers

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global verbosity::1
```

**Expected behavior:**
- Fixture: two path-based projects (`/tmp/proj-a` and `/tmp/proj-b`), one session each
- Output contains:
  ```
  Found 2 sessions:

  /tmp/proj-a: (1 session)
    * session-id-a  Xs ago  (2 entries)

  /tmp/proj-b: (1 session)
    * session-id-b  Xs ago  (2 entries)
  ```
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-15: path header always present at v1 for scope::local single project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local path::{project} verbosity::1
```

**Expected behavior:**
- Fixture: one path project at a known path; `path::` pointing to that project
- stdout contains a line like `/path/to/project: (1 session)` followed by `  * {session-id}`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-16: agent sessions collapsed to count line at v1 without agent:: filter

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global verbosity::1
```

**Expected behavior:**
- Fixture: one project containing 2 main sessions (`session-main-a`, `session-main-b`) and 3 agent sessions (`agent-task-001`, `agent-task-002`, `agent-task-003`)
- Output contains:
  ```
  Found 5 sessions:

  /path/to/project: (5 sessions)
    * session-main-a  Xs ago  (2 entries)
    - session-main-b  Xs ago  (2 entries)
    + 3 agent sessions (last: Xs ago)
  ```
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-17: agent sessions shown individually at v2+

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global verbosity::2
```

**Expected behavior:**
- Fixture: same as INT-16 (2 main + 3 agent sessions in one project)
- All 5 sessions listed individually; no collapse line
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-18: entry count shown per session at v2+

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global verbosity::2
```

**Expected behavior:**
- Fixture: one project and one session containing exactly 4 entries
- Output contains:
  ```
  Found 1 session:

  ~/path/to/project:
    - session-id  (4 entries)
  ```
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-19: agent::1 explicit filter disables collapse at v1

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global verbosity::1 agent::1
```

**Expected behavior:**
- Fixture: same as INT-16 (2 main + 3 agent sessions in one project)
- Agent sessions listed individually when `agent::1` set; no collapse line
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-20: scope::under displays underscore dirs without splitting at `/`

**Command:**
```
clg .projects scope::under path::/tmp/{tempdir}/my_project verbosity::1
```

**Expected behavior:**
- Fixture: create real filesystem directories `/tmp/{tempdir}/my_project/myproject/`; `CLAUDE_STORAGE_ROOT` pointing to a fixture root with a session in the path-encoded `my_project/myproject` project
- stdout contains a line with `my_project` in the project path header; no line contains `wip/core`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-21: scope::global displays hyphen-prefixed topic dir in path header

**Command:**
```
clg .projects scope::global verbosity::1
```

**Expected behavior:**
- Fixture: create real filesystem directory `{tempdir}/src/-default_topic/`; write a session for the project at that path; `CLAUDE_STORAGE_ROOT` and `HOME` set to the temp dir
- stdout path header contains `-default_topic`; no line ends with `src:` (truncated form absent)
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-22: scope::under excludes sibling with underscore-suffix name

**Command:**
```
clg .projects scope::under path::{tempdir}/base
```

**Expected behavior:**
- Fixture: create real filesystem directories `{tempdir}/base/sub/` (child) and `{tempdir}/base_extra/` (sibling); write session `session-it25-child` for the child and `session-it25-sibling` for the sibling; `CLAUDE_STORAGE_ROOT` and `HOME` set to the temp dir
- stdout contains `session-it25-child`; stdout does NOT contain `session-it25-sibling`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-23: scope::relevant excludes sibling with underscore-suffix name

**Command:**
```
clg .projects scope::relevant path::{tempdir}/base_extra
```

**Expected behavior:**
- Fixture: create real filesystem directories `{tempdir}/base/` (sibling) and `{tempdir}/base_extra/` (cwd); write session `session-it26-sibling` for `base` and `session-it26-current` for `base_extra`; `CLAUDE_STORAGE_ROOT` and `HOME` set to the temp dir
- stdout contains `session-it26-current`; stdout does NOT contain `session-it26-sibling`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-24: entry count shown per session at v1

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global verbosity::1
```

**Expected behavior:**
- Fixture: one project and one session containing exactly 4 entries
- Output contains:
  ```
  Found 1 session:

  /path/to/project: (1 session)
    * session-id  Xs ago  (4 entries)
  ```
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-25: limit::N truncates main sessions shown at v1

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global verbosity::1 limit::2
```

**Expected behavior:**
- Fixture: one project containing 5 main sessions
- Truncation hint present with correct count
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-26: zero-byte sessions excluded from v1 display

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global verbosity::1
```

**Expected behavior:**
- Fixture: one project containing one real session (`session-real`, 2 entries) and one zero-byte file (`session-placeholder.jsonl`)
- Real session present; zero-byte placeholder absent
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-27: Summary header format (path, count, age, last-session)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects
```

**Expected behavior:**
- Fixture: project at cwd containing one session with a known UUID and a known number of entries; run from the project directory
- Output contains:
  ```
  Active project  {path}  (N sessions, last active Xd ago)
  Last session:  {8-char-id}  Xd ago  (N entries)
  ```
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-28: Truncation gate — message ≤ 50 chars shown in full

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects
```

**Expected behavior:**
- Fixture: project at cwd containing a session whose last text entry is exactly 40 characters (e.g. `Fix typo in the readme file near line 10`); run from that project
- The `Last message:` section shows the full 40-char string; no `...` appears in the output
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-29: Truncation formula — message > 50 chars as first30...last30

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects
```

**Expected behavior:**
- Fixture: project at cwd containing a session whose last text entry is exactly 60 characters, with distinct known first-30 and last-30 substrings; run from that project
- The `Last message:` section shows `{first30}...{last30}`; the full 60-char source text does NOT appear verbatim
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-30: No sessions in scope shows "No active project found."

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects
```

**Expected behavior:**
- Fixture: empty storage — no session files
- stdout contains `No active project found.`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-31: Explicit scope::local keeps list mode

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local
```

**Expected behavior:**
- Fixture: project at cwd containing at least one session; run from that project
- stdout contains `Found N project` (list-mode header); no `Active project` line
- **⚠️ Maintenance:** The negative check string (`Active project`) must match the current summary-mode header. If the header is renamed, update this test assertion. History: `Active session` (task-007) → `Active project` (task-016)
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-32: Explicit limit::N keeps list mode

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects limit::5
```

**Expected behavior:**
- Fixture: project at cwd containing at least one session; run from that project
- stdout contains `Found N project` (list-mode header); no `Active project` line
- **⚠️ Maintenance:** The negative check string (`Active project`) must match the current summary-mode header. If the header is renamed, update this test assertion. History: `Active session` (task-007) → `Active project` (task-016)
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-33: Family header format (conversations + agents)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local
```

**Expected behavior:**
- Fixture: project containing 1 root session and 3 agent sessions in hierarchical layout (`{uuid}/subagents/`)
- Header contains `conversations` and `agents`; no legacy collapse
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-34: Per-root agent breakdown [N agents: type summary]

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local
```

**Expected behavior:**
- Fixture: project containing 1 root and 3 agents (2×Explore, 1×general-purpose) in hierarchical layout with meta.json sidecars
- Root session line contains `[3 agents: 2×Explore, 1×general-purpose]`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-35: Hierarchical format detection (subagents/ path)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local
```

**Expected behavior:**
- Fixture: 2 root sessions, each with distinct agents in their own `{uuid}/subagents/` directory
- Each root line shows only its own agent count, not the total
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-36: Flat format detection (sessionId linkage)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local
```

**Expected behavior:**
- Fixture: 1 root session and 2 flat agent files; each agent's first JSONL entry has `"sessionId"` matching the root UUID
- Root line shows `[2 agents:` breakdown; flat agents attributed to parent via sessionId
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-37: Orphan family display (root missing)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local
```

**Expected behavior:**
- Fixture: `{uuid}/subagents/agent-*.jsonl` present but NO `{uuid}.jsonl` root file
- Output contains `?` marker on the orphan line
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-38: Childless root (no bracket suffix)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local
```

**Expected behavior:**
- Fixture: 1 root session and 0 agents
- Root line has mtime and entry count but no `[` character
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-39: Meta.json agentType in breakdown

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local
```

**Expected behavior:**
- Fixture: 1 root and 1 agent in hierarchical layout; the agent's `meta.json` contains `{"agentType":"Plan"}`
- Root line contains `Plan` in the bracket breakdown
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-40: Empty/malformed meta.json fallback to "unknown"

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local verbosity::2
```

**Expected behavior:**
- Fixture: 1 root + 1 hierarchical agent, each with 1 JSONL entry
- stdout contains `1 entry` and does NOT contain `1 entries` (correct singular noun for agent entry count)
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-41: verbosity::1 alone stays in summary mode (bug-is-default-verbosity)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects verbosity::1
```

**Expected behavior:**
- Fixture: project at cwd containing at least one session with entries; run from that project
- Same summary block as bare `clg .projects` — NOT a project list:
  ```
  Active project  ~/path/to/project  (N sessions, last active Xd ago)
  Last session:  {8-char-id}  Xd ago  (N entries)

  Last message:
    {message text}
  ```
- stdout does NOT contain `Found N projects:` (list-mode header must be absent)
- **Root Cause (bug-is-default-verbosity):** `is_default` gate in `projects_routine` included `verbosity` in its all-None check (`cmd.get_integer("verbosity").is_none()`). Passing `verbosity::1` returned `Some(1)` instead of `None`, setting `is_default=false` and routing to list mode even though `verbosity::1` is semantically equivalent to the default
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-42: Summary mode shows "Active project" header (task-016)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects
```

**Expected behavior:**
- Fixture: one project at cwd with ≥1 session
- stdout first line starts with `Active project`; `Active session` is absent
- Exit code: 0
- **Source:** `tests/projects_output_format_test.rs::it_summary_mode_shows_active_project_header`

---

### INT-43: Summary mode shows session count aggregate (task-016)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects
```

**Expected behavior:**
- Fixture: one project at cwd with 3 sessions
- stdout contains `sessions,`
- Exit code: 0
- **Source:** `tests/projects_output_format_test.rs::it_summary_mode_shows_session_count`

---

### INT-44: List mode shows projects sorted by recency (task-016)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global
```

**Expected behavior:**
- Fixture: two projects (`proj_alpha` and `proj_beta`) with different file mtimes; `proj_beta` has a newer mtime
- `proj_beta` appears before `proj_alpha` in stdout
- Exit code: 0
- **Source:** `tests/projects_output_format_test.rs::it_list_mode_shows_projects_sorted_by_recency`

---

### INT-45: verbosity::0 outputs project paths only (task-016)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global verbosity::0
```

**Expected behavior:**
- Fixture: one project with ≥1 session
- One line containing the project path; no other output; no `sessions,` or `Found` text
- Exit code: 0
- **Source:** `tests/projects_output_format_test.rs::it_verbosity_0_shows_paths_only`

---

### INT-46: Topic path shown even when topic dir absent from disk

**Command:**
```
clg .projects scope::local
```

**Expected behavior:**
- Fixture: storage root with one project dir `{encoded}--commit` containing one session; the `-commit` filesystem directory does NOT exist under the project path
- Path header contains `/-commit` (topic component appended regardless of disk state)
- Exit code: 0
- **Source:** `tests/projects_path_encoding_test.rs::projects_shows_topic_path_when_topic_dir_absent`

---

### INT-47: Topic path shown when topic dir present on disk

**Command:**
```
clg .projects scope::local
```

**Expected behavior:**
- Fixture: storage root with one project dir `{encoded}--commit` containing one session; the `-commit` filesystem directory DOES exist
- Path header contains `/-commit`
- Exit code: 0
- **Source:** `tests/projects_path_encoding_test.rs::projects_shows_topic_path_when_topic_dir_present`

---

### INT-48: Default-topic path shown when topic dir absent from disk

**Command:**
```
clg .projects scope::local
```

**Expected behavior:**
- Fixture: storage root with one project dir `{encoded}--default-topic` containing one session; the `-default_topic` filesystem directory does NOT exist
- Path header contains `/-default_topic`
- Exit code: 0
- **Source:** `tests/projects_path_encoding_test.rs::projects_shows_default_topic_path_when_topic_dir_absent`

---

### INT-49: Base path shown correctly with no topic suffix

**Command:**
```
clg .projects scope::local
```

**Expected behavior:**
- Fixture: storage root with one plain project dir `{encoded}` (no `--` suffix) containing one session
- Path header shows the decoded base path without any `/-topic` suffix
- Exit code: 0
- **Source:** `tests/projects_path_encoding_test.rs::projects_shows_base_path_with_no_topic`

---

### INT-50: Double-topic key shows both topic components unconditionally

**Command:**
```
clg .projects scope::global verbosity::1
```

**Expected behavior:**
- Fixture: storage root with one project dir `{encoded_base}--default-topic--commit`; topic dirs (`-default_topic`, `-commit`) are NOT created on disk
- Path header contains `/-default_topic` AND `/-commit`
- Exit code: 0
- **Source:** `tests/projects_path_encoding_test.rs::projects_shows_both_topic_components_for_double_topic_key`
