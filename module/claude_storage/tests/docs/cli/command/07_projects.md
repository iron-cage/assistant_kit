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
| INT-14 | default output groups sessions under project path headers | Output Format (plan-004) |
| INT-15 | path header always present in default mode for scope::local single project | Output Format (plan-004) |
| INT-16 | agent sessions collapsed to count line in default mode without agent:: filter | Output Format (plan-004) |
| INT-17 | agent sessions shown individually with show_tree::1 | Output Format (plan-004) |
| INT-18 | entry count shown per session with show_tree::1 | Output Format (plan-004) |
| INT-19 | agent::1 explicit filter disables collapse in default mode | Output Format (plan-004) |
| INT-20 | scope::under displays underscore dirs without splitting at `/` | Underscore Display (issue-029) |
| INT-21 | scope::global displays hyphen-prefixed topic dir in path header | Topic Dir Display (issue-030) |
| INT-22 | scope::under excludes sibling with underscore-suffix name | Sibling Exclusion (issue-031) |
| INT-23 | scope::relevant excludes sibling with underscore-suffix name | Sibling Exclusion (issue-032) |
| INT-24 | entry count shown per session in default mode | Output Format (default mode enhancement) |
| INT-25 | limit::N truncates main sessions shown in default mode | Output Format (default mode enhancement) |
| INT-26 | zero-byte sessions excluded from default mode display | Output Format (default mode enhancement) |
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
| INT-41 | default mode orphan shows `? (orphan)` label (bug-cc-c1) | Family Display |
| INT-42 | show_tree::1 root entry count singular `(1 entry)` | Family Display |
| INT-43 | show_tree::1 agent entry count singular `1 entry` | Family Display |
| INT-41b | default invocation stays in summary mode (no explicit scope/limit) | Summary Mode |
| INT-42b | Summary mode shows "Active project" header (task-016) | Project-Centric Output |
| INT-43b | Summary mode shows session count aggregate (task-016) | Project-Centric Output |
| INT-44 | List mode shows projects sorted by recency (task-016) | Project-Centric Output |
| INT-45 | show_tree::1 outputs tree-indented agent sessions (task-016) | Project-Centric Output |
| INT-46 | Topic path shown even when topic dir absent from disk | Topic Existence Guard (issue-035) |
| INT-47 | Topic path shown when topic dir present on disk | Topic Existence Guard (issue-035) |
| INT-48 | Default-topic path shown when topic dir absent from disk | Topic Existence Guard (issue-035) |
| INT-49 | Base path shown correctly with no topic suffix | Topic Existence Guard (issue-035) |
| INT-50 | Double-topic key shows both topic components unconditionally | Topic Existence Guard (issue-035) |
| INT-51 | scope:: with invalid value rejected | Invalid Parameter Rejection |
| INT-52 | agent:: with non-boolean value rejected | Invalid Parameter Rejection |
| INT-53 | detail::projects shows header line only, no session/family body lines | Detail Level (task-525) |
| INT-54 | detail:: omitted reproduces exact detail::projects output | Detail Level (task-525) |
| INT-55 | detail:: with invalid value rejected | Detail Level (task-525) |
| INT-56 | filter:: narrows to projects whose decoded path contains the substring | Filter Narrowing (task-525) |
| INT-57 | filter:: with no matching project shows empty listing, not an error | Filter Narrowing (task-525) |
| INT-58 | type::uuid narrows to UUID-named projects only | Type Narrowing (task-525) |
| INT-59 | type::path narrows to path-named projects only | Type Narrowing (task-525) |
| INT-60 | type:: with invalid value rejected | Type Narrowing (task-525) |
| INT-61 | project::X ids::1 outputs one conversation ID per line | IDs Scripting Mode (task-525) |
| INT-62 | project::X ids::1 count::1 outputs a single bare integer | IDs Scripting Mode (task-525) |
| INT-63 | ids::1 without required project:: rejected | IDs Scripting Mode (task-525) |
| INT-64 | type:: and filter:: compose under scope::global | Combined Narrowing (task-525) |
| INT-65 | limit::/show_topic:: are no-ops under detail::projects | Detail Level (task-525) |
| INT-65b | show_tree::1 selects the tree layout under detail::projects | Detail Level (task-525) |
| INT-66 | .list's deprecation_message edit does not alter runtime output | `.list` Deprecation (task-525) |
| INT-67 | detail::PROJECTS (mixed-case) matches detail::projects byte-for-byte | Case Insensitivity (task-525) |
| INT-68 | filter::ALPHA-INT68 (mixed-case) matches lowercase-equivalent projects | Case Insensitivity (task-525) |
| OV-1 | Bare .projects renders the terse overview, not session listings | Terse Overview |
| OV-2 | Flat layout emits the LAST/CONV/AGENTS/PROJECT header | Terse Overview |
| OV-3 | Zero agents render as `·`, non-zero as `N ag` | Terse Overview |
| OV-4 | Summary line uses singular nouns at a count of one | Terse Overview |
| OV-5 | A project whose decoded path is absent carries `⚠ gone` | Terse Overview |
| OV-6 | The project matching the process cwd carries the `▸` gutter | Terse Overview |
| OV-7 | show_tree::1 nests projects by directory with tree connectors | Terse Overview |
| OV-8 | Empty storage renders the summary line alone, no header row | Terse Overview |
| OV-9 | detail::sessions still renders the full listing unchanged | Terse Overview |
| OV-10 | Full project paths are printed, never factored to a shared prefix | Terse Overview |
| OV-11 | The tree layout marks an absent decoded path `⚠ gone` too | Terse Overview |
| OV-12 | A single-child directory run collapses into one tree node | Terse Overview |

## Test Coverage Summary

- Summary Mode (default): 1 test (INT-1)
- Summary Mode: 5 tests (INT-27–INT-30, INT-41b)
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
- Output Format (default mode enhancement): 3 tests (INT-24, INT-25, INT-26)
- Family Display: 11 tests (INT-33 through INT-40, INT-41 through INT-43)
- Project-Centric Output (task-016): 4 tests (INT-42b, INT-43b, INT-44, INT-45)
- Topic Existence Guard (issue-035): 5 tests (INT-46 through INT-50)
- Invalid Parameter Rejection: 2 tests (INT-51, INT-52)
- Detail Level (task-525): 5 tests (INT-53, INT-54, INT-55, INT-65, INT-65b)
- Filter Narrowing (task-525): 2 tests (INT-56, INT-57)
- Type Narrowing (task-525): 3 tests (INT-58, INT-59, INT-60)
- IDs Scripting Mode (task-525): 3 tests (INT-61, INT-62, INT-63)
- Combined Narrowing (task-525): 1 test (INT-64)
- `.list` Deprecation (task-525): 1 test (INT-66)
- Case Insensitivity (task-525): 2 tests (INT-67, INT-68)
- Terse Overview: 12 tests (OV-1 through OV-12) — `tests/projects_overview_test.rs`

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

### INT-14: default output groups sessions under project path headers

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global```

**Expected behavior:**
- Fixture: two path-based projects (`/tmp/proj-a` and `/tmp/proj-b`), one session each
- Output contains:
  ```
  Found 2 projects:

  /tmp/proj-a: (1 conversation)
    * session-id-a  Xs ago  (2 entries)

  /tmp/proj-b: (1 conversation)
    * session-id-b  Xs ago  (2 entries)
  ```
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-15: path header always present in default mode for scope::local single project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local path::{project}```

**Expected behavior:**
- Fixture: one path project at a known path; `path::` pointing to that project
- stdout contains a line like `/path/to/project: (1 conversation)` followed by `  * {session-id}`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-16: agent sessions collapsed to count line in default mode without agent:: filter

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global```

**Expected behavior:**
- Fixture: one project containing 2 main sessions (`session-main-a`, `session-main-b`) and 3 agent sessions (`agent-task-001`, `agent-task-002`, `agent-task-003`)
- Output contains:
  ```
  Found 5 projects:

  /path/to/project: (5 conversations)
    * session-main-a  Xs ago  (2 entries)
    - session-main-b  Xs ago  (2 entries)
    + 3 agent sessions (last: Xs ago)
  ```
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-17: agent sessions shown individually with show_tree::1

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global show_tree::1
```

**Expected behavior:**
- Fixture: same as INT-16 (2 main + 3 agent sessions in one project)
- All 5 sessions listed individually; no collapse line
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-18: entry count shown per session with show_tree::1

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global show_tree::1
```

**Expected behavior:**
- Fixture: one project and one session containing exactly 4 entries
- Output contains:
  ```
  Found 1 project:

  ~/path/to/project:
    - session-id  (4 entries)
  ```
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-19: agent::1 explicit filter disables collapse in default mode

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global agent::1
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
clg .projects scope::under path::/tmp/{tempdir}/my_project```

**Expected behavior:**
- Fixture: create real filesystem directories `/tmp/{tempdir}/my_project/myproject/`; `CLAUDE_STORAGE_ROOT` pointing to a fixture root with a session in the path-encoded `my_project/myproject` project
- stdout contains a line with `my_project` in the project path header; no line contains `wip/core`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-21: scope::global displays hyphen-prefixed topic dir in path header

**Command:**
```
clg .projects scope::global```

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

### INT-24: entry count shown per session in default mode

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global```

**Expected behavior:**
- Fixture: one project and one session containing exactly 4 entries
- Output contains:
  ```
  Found 1 project:

  /path/to/project: (1 conversation)
    * session-id  Xs ago  (4 entries)
  ```
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-25: limit::N truncates main sessions shown in default mode

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global limit::2
```

**Expected behavior:**
- Fixture: one project containing 5 main sessions
- Truncation hint present with correct count
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-26: zero-byte sessions excluded from default mode display

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global```

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
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local show_tree::1
```

**Expected behavior:**
- Fixture: 1 root + 1 hierarchical agent, each with 1 JSONL entry
- stdout contains `1 entry` and does NOT contain `1 entries` (correct singular noun for agent entry count)
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-41: Default mode orphan shows `? (orphan)` label (bug-cc-c1)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local path::{project-path}
```

**Expected behavior:**
- Fixture: one flat agent session whose parent session ID does not exist in storage (orphan family)
- Output contains `? (orphan)` as the family label
- Exit code: 0
- **Source:** `tests/cli_cmd_projects_summary_test.rs::int_41_v1_orphan_shows_orphan_label`

---

### INT-42: `show_tree::1` root entry count singular — `(1 entry)` not `(1 entries)`

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local path::{project-path} show_tree::1
```

**Expected behavior:**
- Fixture: project with exactly 1 session containing exactly 1 entry
- Output contains `(1 entry)` (singular noun)
- Output does NOT contain `(1 entries)`
- Exit code: 0
- **Source:** `tests/cli_cmd_projects_summary_test.rs::int_42_show_tree_root_entry_count_singular`

---

### INT-43: `show_tree::1` agent entry count singular — `1 entry` not `1 entries`

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::local path::{project-path} show_tree::1
```

**Expected behavior:**
- Fixture: project with 1 root session and 1 hierarchical agent subagent, each with exactly 1 entry
- Output contains `1 entry` (singular) on the agent line
- Output does NOT contain `1 entries`
- Exit code: 0
- **Source:** `tests/cli_cmd_projects_summary_test.rs::int_43_show_tree_agent_entry_count_singular`

---

### INT-41b: default invocation stays in summary mode (no explicit scope/limit)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects
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
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-42b: Summary mode shows "Active project" header (task-016)

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

### INT-43b: Summary mode shows session count aggregate (task-016)

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

### INT-45: show_tree::1 outputs tree-indented agent sessions (task-016)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global show_tree::1
```

**Expected behavior:**
- Fixture: one project with 1 main session and ≥1 agent session
- Agent sessions appear tree-indented under their parent root session (with `├─`/`└─` connectors)
- No collapse line (`+ N agent sessions`)
- Exit code: 0
- **Source:** `tests/projects_output_format_test.rs::it_20_agent_sessions_shown_individually_at_v2`

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
clg .projects scope::global```

**Expected behavior:**
- Fixture: storage root with one project dir `{encoded_base}--default-topic--commit`; topic dirs (`-default_topic`, `-commit`) are NOT created on disk
- Path header contains `/-default_topic` AND `/-commit`
- Exit code: 0
- **Source:** `tests/projects_path_encoding_test.rs::projects_shows_both_topic_components_for_double_topic_key`

---

### INT-51: scope:: with invalid value rejected

**Command:**
```
clg .projects scope::badvalue
```

**Expected behavior:**
- `badvalue` is not a valid option for `scope::` (accepted: `local`, `under`, `relevant`, `global`, `around`)
- Error message on stderr naming the invalid value
- No project output on stdout
- Exit code: 1
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-52: agent:: with non-boolean value rejected

**Command:**
```
clg .projects agent::invalid
```

**Expected behavior:**
- `invalid` is not a valid boolean value for `agent::` (accepted: `0`, `1`)
- Error message on stderr describing the argument error
- No project output on stdout
- Exit code: 1
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-53: detail::projects shows header line only, no session/family body lines

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global detail::projects
```

**Expected behavior:**
- Fixture: two projects in scope — one with 2 conversations and 12 total agents, one with 1 plain session
- Output:
  ```
  2 projects · 3 conversations · 12 agents

    LAST      CONV  AGENTS  PROJECT
    2h ago  2 conv   12 ag  ~/path/to/project-a
    1d ago  1 conv       ·  ~/path/to/project-b
  ```
- Output does NOT contain any `*`/`-` session lines or `[N agents: ...]` breakdowns
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-54: detail:: omitted reproduces exact detail::projects output

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global
```

**Expected behavior:**
- Fixture: same as INT-53
- stdout is byte-for-byte identical to the same invocation with explicit `detail::projects` appended — `detail::` defaults to `projects`, not `sessions`; this is a regression guard against a wrong default. It pinned the opposite default before the terse overview became the primary view: `sessions` was the default only to preserve `.list`'s behavior through its absorption into `.projects`, and that rationale expired with `.list`
- stdout does NOT contain session ids
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-55: detail:: with invalid value rejected

**Command:**
```
clg .projects detail::bogus
```

**Expected behavior:**
- `bogus` is not a valid option for `detail::` (accepted: `projects`, `sessions`)
- stderr contains the exact text `detail must be projects|sessions, got bogus`
- No project output on stdout
- Exit code: 1
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-56: filter:: narrows to projects whose decoded path contains the substring

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global filter::alpha
```

**Expected behavior:**
- Fixture: three projects in scope with decoded paths containing `alpha`, `beta`, and `gamma` respectively
- stdout lists only the `alpha` project; the `beta` and `gamma` projects are absent
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-57: filter:: with no matching project shows empty listing, not an error

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global filter::nonexistent-substring
```

**Expected behavior:**
- Fixture: one or more projects in scope, none of whose decoded paths contain the filter substring
- stdout contains `Found 0 projects:`
- Exit code: 0 (an empty result is valid, not an error)
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-58: type::uuid narrows to UUID-named projects only

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global type::uuid
```

**Expected behavior:**
- Fixture: scope containing both a UUID-identified project and a path-identified project
- stdout lists only the UUID-identified project; the path-identified project is absent
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-59: type::path narrows to path-named projects only

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global type::path
```

**Expected behavior:**
- Fixture: same as INT-58 (one UUID-identified project, one path-identified project)
- stdout lists only the path-identified project; the UUID-identified project is absent
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-60: type:: with invalid value rejected

**Command:**
```
clg .projects type::bogus
```

**Expected behavior:**
- `bogus` is not a valid option for `type::` (accepted: `uuid`, `path`, `all`)
- stderr contains the exact text `type must be uuid|path|all, got bogus`
- No project output on stdout
- Exit code: 1
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-61: project::X ids::1 outputs one conversation ID per line

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects project::abc123 ids::1
```

**Expected behavior:**
- Fixture: project `abc123` containing N distinct root conversations (some with agent children)
- stdout contains exactly N lines, each a bare conversation ID; no path headers, no `Found` line, no session-count aggregates
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-62: project::X ids::1 count::1 outputs a single bare integer

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects project::abc123 ids::1 count::1
```

**Expected behavior:**
- Fixture: same project as INT-61, with N root conversations
- stdout is exactly the single line `N` — no conversation IDs, no other text
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-63: ids::1 without required project:: rejected

**Command:**
```
clg .projects ids::1
```

**Expected behavior:**
- `ids::1` requires `project::`; omitting it is an argument error
- stderr contains a non-empty error naming `project::` as required
- No conversation IDs on stdout
- Exit code: 1
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-64: type:: and filter:: compose under scope::global

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global type::path filter::alpha
```

**Expected behavior:**
- Fixture: scope containing a path-identified project matching `alpha`, a path-identified project matching `beta`, and a UUID-identified project whose decoded path also matches `alpha`
- stdout lists only the path-identified `alpha` project — the path-identified `beta` project is excluded by `filter::`, the UUID-identified `alpha` project is excluded by `type::path`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-65: limit::/show_topic:: are no-ops under detail::projects

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global detail::projects limit::1 show_topic::1
```

**Expected behavior:**
- Fixture: same as INT-53
- stdout is byte-for-byte identical to the same command without `limit::1 show_topic::1` — `limit::1` does not truncate the project list, and `show_topic::1` has no visible effect since no session lines exist to annotate
- `show_tree::` was in this no-op set until the terse overview gained a tree layout; it is now a live parameter here, covered by INT-65b
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-65b: show_tree::1 selects the tree layout under detail::projects

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global detail::projects show_tree::1
```

**Expected behavior:**
- Fixture: two sibling projects under a common parent directory (nesting is required — a lone project collapses to a single top-level node, which by construction draws no connector)
- stdout differs from the same command without `show_tree::1`
- Flat output contains no `├`/`└` connectors; tree output contains at least one
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md); test: `int_65b_show_tree_selects_tree_layout_under_detail_projects`

---

### INT-66: .list's deprecation_message edit does not alter runtime output

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .list
```

**Expected behavior:**
- Fixture: any fixture producing non-trivial `.list` output (e.g. one project with sessions)
- stdout is byte-for-byte identical to `.list`'s pre-task output — `unilang.commands.yaml`'s `deprecation_message` field is metadata consumed only by the `--help` generator and build-time registry code, never by the dispatch/execution path (part of the same `.list`→`.projects` absorption change as INT-53 through INT-65; see [command/02_list.md](../../../../docs/cli/command/02_list.md) for `.list`'s own deprecated-status documentation)
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md)

---

### INT-67: detail::PROJECTS (mixed-case) matches detail::projects byte-for-byte

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global detail::PROJECTS
```

**Expected behavior:**
- Fixture: one hierarchical family project (root + 2 agent sessions) plus one plain path-based project (single session)
- stdout is byte-for-byte identical to the same command run with `detail::projects` (lowercase) — `validate_detail_level` calls `.to_lowercase()` on the raw value before matching against `projects`/`sessions`
- Sanity: the summary line still reports `2 projects`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md); scope amendment to task-525 (case-insensitivity regression coverage for `detail::`); test: `int_67_detail_uppercase_matches_lowercase`

---

### INT-68: filter::ALPHA-INT68 (mixed-case) matches lowercase-equivalent projects

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global filter::ALPHA-INT68
```

**Expected behavior:**
- Fixture: two path-based projects whose decoded paths contain `alpha-int68` and `beta-int68` respectively
- stdout includes only the `alpha-int68` project; the `beta-int68` project is absent — both the supplied filter substring and the decoded display path are lowercased before the `contains` check, so casing never affects the match
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md); scope amendment to task-525 (case-insensitivity regression coverage for `filter::`); test: `int_68_filter_uppercase_matches_lowercase`

---

## Terse Overview (OV-1 – OV-12)

Rendering cases for `detail::projects` — the default view since the terse overview
became `.projects`' primary answer. Implemented in `tests/projects_overview_test.rs`
against `src/cli/projects_overview.rs`. OV-9 is the only case here that exercises
`detail::sessions`, as a guard that the terse renderer did not leak into that path.

The conditional `STATUS` column and the `detail::sessions` state tag are the one
part of this rendering not covered here: they depend on the real process table,
so their cases live with the parameter that shares that dependency — see
[`param/44_live.md`](../param/44_live.md) (EC-6, EC-7) for the absent-affordance
contract, and `src/cli/liveness.rs`'s unit tests for the positive side.

---

### OV-1: Bare .projects renders the terse overview, not session listings

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global
```

**Expected behavior:**
- Fixture: one project with a root session plus one agent session
- stdout contains the summary line (`1 project · …`)
- stdout does NOT contain `Found 1 project:`, the root session id, or the agent session id
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md); test: `ov_1_bare_projects_renders_terse_overview`

---

### OV-2: Flat layout emits the LAST/CONV/AGENTS/PROJECT header

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global
```

**Expected behavior:**
- Fixture: one path-based project with a single session
- stdout contains all four column names, and the header line precedes every project row
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md); test: `ov_2_flat_layout_emits_column_header`

---

### OV-3: Zero agents render as `·`, non-zero as `N ag`

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global
```

**Expected behavior:**
- Fixture: one agentless project plus one project with a single agent
- stdout contains `·` for the agentless row and `1 ag` for the other; `0 ag` never appears
- Rationale: a column of zeroes is noise in a list where most projects never spawn an agent
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md); test: `ov_3_zero_agents_render_as_middot`

---

### OV-4: Summary line uses singular nouns at a count of one

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global
```

**Expected behavior:**
- Fixture: one project with one session and no agents
- First line reads `1 project · 1 conversation` — never `1 projects` or `1 conversations`
- The agents segment is omitted entirely rather than rendered as `0 agents`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md); test: `ov_4_summary_line_uses_singular_nouns`

---

### OV-5: A project whose decoded path is absent carries `⚠ gone`

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global
```

**Expected behavior:**
- Fixture: a storage entry for a directory that was never created on disk — the deleted-scratch-directory case
- stdout contains `⚠ gone` on that row, and still shows the path itself
- Rationale: encoding is lossy (`/`, `_`, and `.` all collapse to `-`), so a decoded path is only trustworthy while the directory it names exists to disambiguate it
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md); test: `ov_5_absent_decoded_path_marked_gone`

---

### OV-6: The project matching the process cwd carries the `▸` gutter

**Command:**
```
cd <project> && CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global
```

**Expected behavior:**
- Fixture: one project whose directory exists on disk
- Run with cwd inside that project: stdout contains `▸`
- Run with cwd matching no listed project: stdout contains no `▸`
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md); test: `ov_6_cwd_project_carries_gutter_marker`

---

### OV-7: show_tree::1 nests projects by directory with tree connectors

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global show_tree::1
```

**Expected behavior:**
- Fixture: two sibling projects (`parent/alpha`, `parent/beta`)
- stdout contains `├`/`└` connectors and both leaf names
- The shared parent segment appears exactly once, as a node — not repeated per row
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md); test: `ov_7_show_tree_nests_projects_by_directory`

---

### OV-8: Empty storage renders the summary line alone, no header row

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/empty-fixture clg .projects scope::global
```

**Expected behavior:**
- Fixture: a storage root containing an empty `projects/` directory
- stdout reports `0 projects` and contains neither `LAST` nor `PROJECT` — a column header over zero rows is a phantom table
- Exit code: 0, not an error
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md); test: `ov_8_empty_storage_renders_summary_only`

---

### OV-9: detail::sessions still renders the full listing unchanged

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global detail::sessions
```

**Expected behavior:**
- Fixture: one path-based project with a single session
- stdout contains `Found 1 project:` and the session id
- stdout contains neither the terse column header nor the terse summary line's `·` separator
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md); test: `ov_9_detail_sessions_renders_full_listing`

---

### OV-10: Full project paths are printed, never factored to a shared prefix

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global
```

**Expected behavior:**
- Fixture: two projects sharing an ancestor directory (`shared/alpha`, `shared/beta`)
- The shared ancestor appears in both rows in full — not factored out into a `base` line with relative row labels
- Rationale: a project path is the command's primary output and must stay usable in `cd`, `grep`, and `project::`. Prefix factoring is `show_tree::1`'s job, where nesting carries the shared segment without truncating any row
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md); test: `ov_10_flat_layout_prints_full_paths`

---

### OV-11: The tree layout marks an absent decoded path `⚠ gone` too

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global show_tree::1
```

**Expected behavior:**
- Fixture: two siblings under one parent (`parent/live`, `parent/vanished`), only `live` created on disk
- Tree connectors are drawn; exactly one line carries `⚠ gone`, and it is the `vanished` row
- Rationale: `render_tree` resolves each node back to its row rather than iterating rows, so it computes the marker on a code path OV-5 never reaches
- The fixture root must be dot-free. `encode_path` collapses `.` to `-` exactly as it does `/`, so under a `.tmpXXXX` root the absent sibling has no directory left to disambiguate against, decodes to a mangled flat name, and never nests — real decoder behavior, avoided rather than asserted around
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md); test: `ov_11_tree_layout_marks_absent_decoded_path_gone`

---

### OV-12: A single-child directory run collapses into one tree node

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .projects scope::global show_tree::1
```

**Expected behavior:**
- Fixture: one project under a three-deep single-child chain (`a/b/c/leaf`)
- The whole run occupies a single line labelled `a/b/c/leaf`; no connector is drawn, since a collapsed chain has no branch point
- Rationale: without `collapse`, a deeply-nested project draws one level per directory carrying no information, making the tree taller than the flat table it compresses. OV-7 proves nesting at a branch point but never exercises a run with nothing to branch on
- Exit code: 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md); test: `ov_12_single_child_chain_collapses_to_one_node`
