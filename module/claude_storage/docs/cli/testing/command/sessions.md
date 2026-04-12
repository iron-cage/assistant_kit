# Command :: `.sessions`

Integration tests for the `.sessions` command. Tests verify summary mode output (default), scope semantics, path anchoring, filter behavior, and exit code contracts.

**Source:** [commands.md#command--8-sessions](../../commands.md#command--8-sessions)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Default (no args) shows active-session summary | Summary Mode (default) |
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
| IT-17 | v1 output groups sessions under project path headers | Output Format (plan-004) |
| IT-18 | path header always present at v1 for scope::local single project | Output Format (plan-004) |
| IT-19 | agent sessions collapsed to count line at v1 without agent:: filter | Output Format (plan-004) |
| IT-20 | agent sessions shown individually at v2+ | Output Format (plan-004) |
| IT-21 | entry count shown per session at v2+ | Output Format (plan-004) |
| IT-22 | agent::1 explicit filter disables collapse at v1 | Output Format (plan-004) |
| IT-23 | scope::under displays underscore dirs without splitting at `/` | Underscore Display (issue-029) |
| IT-24 | scope::global displays hyphen-prefixed topic dir in path header | Topic Dir Display (issue-030) |
| IT-25 | scope::under excludes sibling with underscore-suffix name | Sibling Exclusion (issue-031) |
| IT-26 | scope::relevant excludes sibling with underscore-suffix name | Sibling Exclusion (issue-032) |
| IT-27 | entry count shown per session at v1 | Output Format (v1 enhancement) |
| IT-28 | limit::N truncates main sessions shown at v1 | Output Format (v1 enhancement) |
| IT-29 | zero-byte sessions excluded from v1 display | Output Format (v1 enhancement) |
| IT-30 | Summary header format (id, age, count, path) | Summary Mode |
| IT-31 | Truncation gate — message ≤ 50 chars shown in full | Summary Mode |
| IT-32 | Truncation formula — message > 50 chars as first30...last30 | Summary Mode |
| IT-33 | No sessions in scope shows "No active session found." | Summary Mode |
| IT-34 | Explicit scope::local keeps list mode | Filter Passthrough |
| IT-35 | Explicit limit::N keeps list mode | Filter Passthrough |
| IT-36 | Family header format (conversations + agents) | Family Display |
| IT-37 | Per-root agent breakdown [N agents: type summary] | Family Display |
| IT-38 | Hierarchical format detection (subagents/ path) | Family Display |
| IT-39 | Flat format detection (sessionId linkage) | Family Display |
| IT-40 | Orphan family display (root missing) | Family Display |
| IT-41 | Childless root (no bracket suffix) | Family Display |
| IT-42 | Meta.json agentType in breakdown | Family Display |
| IT-43 | Empty/malformed meta.json fallback to "unknown" | Family Display |
| IT-44 | v1 orphan shows `? (orphan)` label (bug-cc-c1) | Family Display |
| IT-45 | v2 root entry count singular `(1 entry)` | Family Display |
| IT-46 | v2 agent entry count singular `1 entry` | Family Display |
| IT-47 | verbosity::1 alone stays in summary mode (bug-is-default-verbosity) | Summary Mode |

## Test Coverage Summary

- Summary Mode (default): 1 test (IT-1)
- Summary Mode: 5 tests (IT-30–IT-33, IT-47)
- Filter Passthrough: 2 tests (IT-34–IT-35)
- Scope Behavior: 3 tests (IT-2, IT-3, IT-4)
- Path Anchoring: 1 test (IT-5)
- Filter Behavior: 2 tests (IT-6, IT-7)
- Exit Codes: 1 test (IT-8)
- Underscore Path (issue-024): 5 tests (IT-9 through IT-13)
- Output Format (plan-004): 6 tests (IT-17 through IT-22)
- Underscore Display (issue-029): 1 test (IT-23)
- Topic Dir Display (issue-030): 1 test (IT-24)
- Sibling Exclusion (issue-031): 1 test (IT-25)
- Sibling Exclusion (issue-032): 1 test (IT-26)
- Output Format (v1 enhancement): 3 tests (IT-27, IT-28, IT-29)
- Family Display: 11 tests (IT-36 through IT-43, IT-44 through IT-46)

## Test Cases

### IT-1: Default (no args) shows active-session summary

**Goal:** Verify that bare `.sessions` with no arguments outputs a single-session summary block — not a session list. The summary shows the most-recent session's ID (first 8 chars), age, entry count, project path relative to cwd, and last message text.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at `/home/user1/pro/alpha` containing at least one session with entries. Run from `/home/user1/pro/alpha`.
**Command:** `clg .sessions`
**Expected Output:**
```
Active session  {8-char-id}  Xd ago  N entries
Project  ~/pro/alpha

Last message:
  {message text or truncated form}
```
stdout does NOT contain `Found N sessions:` (list-mode header absent).
**Verification:**
- Exit code is 0
- stdout first line contains `Active session`
- stdout contains a `Project` line with the project path
- stdout contains `Last message:` header
- stdout does NOT contain `Found N sessions:`
**Pass Criteria:** exit 0 + summary header present + `Found N sessions:` absent

**Source:** [commands.md](../../commands.md)

---

### IT-2: scope::relevant includes ancestor project sessions

**Goal:** Verify that `scope::relevant` walks up the ancestor chain from cwd and includes sessions from all ancestor projects.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b/c`, `/a/b`, and `/a`. Run from `/a/b/c`.
**Command:** `clg .sessions scope::relevant`
**Expected Output:** stdout lists sessions from all three projects: `/a/b/c`, `/a/b`, and `/a`.
**Verification:**
- Exit code is 0
- Sessions from `/a/b/c` are listed (current project)
- Sessions from `/a/b` are listed (parent project)
- Sessions from `/a` are listed (grandparent project)
- Sessions from unrelated paths are absent
**Pass Criteria:** exit 0 + sessions from all ancestor-chain projects present

**Source:** [commands.md](../../commands.md)

---

### IT-3: scope::under includes descendant project sessions

**Goal:** Verify that `scope::under` returns sessions from all projects nested beneath the base path.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b`, `/a/b/c`, `/a/b/c/d`, and `/z` (unrelated). Run from `/a/b`.
**Command:** `clg .sessions scope::under`
**Expected Output:** stdout lists sessions from `/a/b`, `/a/b/c`, and `/a/b/c/d`; not from `/z`.
**Verification:**
- Exit code is 0
- Sessions from `/a/b` are listed (root of subtree)
- Sessions from `/a/b/c` are listed (child)
- Sessions from `/a/b/c/d` are listed (grandchild)
- Sessions from `/z` are absent (outside subtree)
**Pass Criteria:** exit 0 + sessions from all descendant projects present; unrelated projects absent

**Source:** [commands.md](../../commands.md)

---

### IT-4: scope::global returns all sessions regardless of path

**Goal:** Verify that `scope::global` returns sessions from every project in storage, ignoring any path context.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b`, `/c/d`, and `/e/f`. Run from `/a/b`.
**Command:** `clg .sessions scope::global`
**Expected Output:** stdout lists sessions from all three projects.
**Verification:**
- Exit code is 0
- Sessions from `/a/b` are listed
- Sessions from `/c/d` are listed (unrelated to cwd)
- Sessions from `/e/f` are listed (unrelated to cwd)
- Total session count matches the sum of all fixture projects
**Pass Criteria:** exit 0 + sessions from all projects in storage

**Source:** [commands.md](../../commands.md)

---

### IT-5: path:: overrides cwd as scope anchor

**Goal:** Verify that `path::` replaces cwd as the scope anchor so scope resolution is performed relative to the specified path rather than the running directory.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/a/b/c`, `/a/b`, and `/a`. Run from `/tmp` (no project there).
**Command:** `clg .sessions scope::local path::/a/b/c`
**Expected Output:** Sessions from the project at `/a/b/c` only; cwd (`/tmp`) has no effect.
**Verification:**
- Exit code is 0
- Sessions from `/a/b/c` are listed
- Sessions from `/a/b` and `/a` are absent (local scope, not relevant)
- Output is the same as running from `/a/b/c` with `scope::local` and no `path::`
**Pass Criteria:** exit 0 + path parameter used as anchor instead of cwd

**Source:** [commands.md](../../commands.md)

---

### IT-6: session:: filter narrows results

**Goal:** Verify that `session::` filters out sessions whose ID does not contain the given substring.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing sessions `-commit.jsonl` and `-default_topic.jsonl`. Run from that project.
**Command:** `clg .sessions session::commit`
**Expected Output:** stdout lists only sessions matching "commit" in their ID; `-default_topic` session is absent.
**Verification:**
- Exit code is 0
- Session `-commit` is listed
- Session `-default_topic` is not listed
- Session count in output is less than without the filter
**Pass Criteria:** exit 0 + only sessions with "commit" in ID appear

**Source:** [commands.md](../../commands.md)

---

### IT-7: min_entries:: filter excludes short sessions

**Goal:** Verify that `min_entries::N` excludes sessions with fewer than N entries from the results.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing: one session with 3 entries and one session with 15 entries. Run from that project.
**Command:** `clg .sessions min_entries::10`
**Expected Output:** stdout lists only the session with 15 entries; the 3-entry session is absent.
**Verification:**
- Exit code is 0
- The 15-entry session is listed
- The 3-entry session is not listed
- Output is a subset of the unfiltered result
**Pass Criteria:** exit 0 + only sessions meeting the entry count threshold appear

**Source:** [commands.md](../../commands.md)

---

### IT-8: No matching sessions exits with code 0

**Goal:** Verify that `.sessions` exits with code `0` even when no sessions match the scope — empty results are not an error.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (empty storage — no projects). Run from any directory.
**Command:** `clg .sessions scope::global`
**Expected Output:** stdout is empty or contains a "no sessions found" indication; exit code is 0.
**Verification:**
- `$?` is `0` (empty results are not an error)
- `$?` is NOT `2` (empty storage is not a storage read error)
- stderr is empty
- stdout is empty or contains a benign "no sessions" message
**Pass Criteria:** exit 0 + no error on stderr for empty/no-match storage

**Source:** [commands.md](../../commands.md)

---

### IT-9: scope::local finds project when path contains underscores

**Goal:** Verify that `scope::local` returns sessions for a project whose path contains underscores (regression for issue-024: encode/decode lossy round-trip caused silent 0-result return).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at `/home/user1/wip_core`. Run from `/home/user1/wip_core`.
**Command:** `clg .sessions scope::local`
**Expected Output:** stdout lists the session from `/home/user1/wip_core`; exit code 0.
**Verification:**
- Exit code is 0
- Session from the underscore-path project is listed
- stdout is non-empty (session found, not 0 results)
**Pass Criteria:** exit 0 + session from underscore-path project appears in output

**Source:** [commands.md](../../commands.md)

---

### IT-10: scope::under finds subtree when base path has underscores

**Goal:** Verify that `scope::under` returns sessions from child projects when the base path contains underscores (regression for issue-024).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/home/user1/wip_core` and `/home/user1/wip_core/child`. Run from `/home/user1/wip_core`.
**Command:** `clg .sessions scope::under`
**Expected Output:** stdout lists sessions from both `/home/user1/wip_core` and `/home/user1/wip_core/child`; exit code 0.
**Verification:**
- Exit code is 0
- Sessions from the base underscore-path project are listed
- Sessions from the child project are listed
- stdout is non-empty
**Pass Criteria:** exit 0 + sessions from all underscore-base subtree projects present

**Source:** [commands.md](../../commands.md)

---

### IT-11: scope::relevant finds ancestor when path has underscores

**Goal:** Verify that `scope::relevant` finds an ancestor project when the ancestor path contains underscores (regression for issue-024).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `/home/user1/wip_core` (ancestor) and `/home/user1/wip_core/sub/child` (current). Run from `/home/user1/wip_core/sub/child`.
**Command:** `clg .sessions scope::relevant`
**Expected Output:** stdout lists sessions from both projects (current + ancestor with underscores); exit code 0.
**Verification:**
- Exit code is 0
- Sessions from the underscore-path ancestor are listed
- Sessions from the child project are listed
- stdout is non-empty
**Pass Criteria:** exit 0 + sessions from ancestor with underscore path appear

**Source:** [commands.md](../../commands.md)

---

### IT-12: scope::relevant finds topic-scoped ancestor with underscores

**Goal:** Verify that `scope::relevant` resolves ancestor projects that have both underscores in the path AND a topic suffix (e.g., `-default_topic`). Topic suffix uses `--` separator; ancestor stripping must not confuse `-` (path separator) with `--` (topic separator).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at `/home/user1/wip_core` with topic `default_topic` (storage dir ends in `--default-topic`). Run from `/home/user1/wip_core/child`.
**Command:** `clg .sessions scope::relevant`
**Expected Output:** stdout lists sessions from the topic-scoped ancestor project; exit code 0.
**Verification:**
- Exit code is 0
- Sessions from the topic-scoped ancestor (with underscore + topic suffix) are listed
- stdout is non-empty
**Pass Criteria:** exit 0 + topic-scoped underscore-path ancestor sessions appear

**Source:** [commands.md](../../commands.md)

---

### IT-13: scope::under with multiple underscore components finds nested projects

**Goal:** Verify that `scope::under` finds child projects when the base path contains **multiple** underscore components (e.g., `my_project/sub_module`). This is a more complex case than IT-10 (single underscore) and tests that all intermediate encoded components are matched correctly.

**Note — encoding limitation (superseded):** The original ambiguity between sibling `my_project_x` and child `my_project/x` (both encode to `my_project-x`) was resolved by the two-stage predicate in issue-031 (TSK-060): string prefix is fast-reject only; `decode_path_via_fs` + `Path::starts_with` (component-wise) correctly excludes siblings. See IT-25 for the regression test.

**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with projects at `root/my_project/sub_module` (base), `root/my_project/sub_module/feature_x` (child), and `root/other_project` (unrelated). Run with `path::root/my_project/sub_module`.
**Command:** `clg .sessions scope::under path::root/my_project/sub_module`
**Expected Output:** stdout lists sessions from base and child; sessions from `root/other_project` are absent.
**Verification:**
- Exit code is 0
- Sessions from `root/my_project/sub_module` are listed (exact match)
- Sessions from `root/my_project/sub_module/feature_x` are listed (child)
- Sessions from `root/other_project` are absent (unrelated prefix)
**Pass Criteria:** exit 0 + multi-underscore-component base + child sessions both appear; unrelated session absent

**Source:** [commands.md](../../commands.md)

---

### IT-17: v1 output groups sessions under project path headers

**Goal:** Verify that `verbosity::1` output groups sessions under human-readable `~/path/to/project: (N sessions)` headers rather than listing them flat.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with two path-based projects (e.g., `/tmp/proj-a` and `/tmp/proj-b`), one session each.
**Command:** `clg .sessions scope::global verbosity::1`
**Expected Output:**
```
Found 2 sessions:

/tmp/proj-a: (1 session)
  * session-id-a  Xs ago  (2 entries)

/tmp/proj-b: (1 session)
  * session-id-b  Xs ago  (2 entries)
```
**Verification:**
- Exit code is 0
- stdout contains at least one header line with `:` that includes `/` or `~`
- Each session ID appears indented below its project header
**Pass Criteria:** exit 0 + path headers present + sessions grouped below them

**Source:** [commands.md](../../commands.md)

---

### IT-18: path header always present at v1 for scope::local single project

**Goal:** Verify that the project path header appears at verbosity 1 even for a single matched project (scope::local).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with one path project at a known path. Run with `path::` pointing to that project.
**Command:** `clg .sessions scope::local path::{project} verbosity::1`
**Expected Output:** stdout contains a line like `/path/to/project: (1 session)` followed by `  * {session-id}`.
**Verification:**
- Exit code is 0
- stdout contains a header line with `:` that includes `/` or `~`
**Pass Criteria:** exit 0 + path header present

**Source:** [commands.md](../../commands.md)

---

### IT-19: agent sessions collapsed to count line at v1 without agent:: filter

**Goal:** Verify that at verbosity 1 with no `agent::` filter, agent sessions (IDs starting with `agent-`) are collapsed to a `+ N agent session(s)` count line rather than listed individually.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with one project containing 2 main sessions (`session-main-a`, `session-main-b`) and 3 agent sessions (`agent-task-001`, `agent-task-002`, `agent-task-003`).
**Command:** `clg .sessions scope::global verbosity::1`
**Expected Output:**
```
Found 5 sessions:

/path/to/project: (5 sessions)
  * session-main-a  Xs ago  (2 entries)
  - session-main-b  Xs ago  (2 entries)
  + 3 agent sessions (last: Xs ago)
```
**Verification:**
- Exit code is 0
- stdout does NOT contain individual agent session IDs
- stdout contains `3 agent`
- stdout contains both main session IDs
- stdout contains mtime hint on the collapse line
**Pass Criteria:** exit 0 + agents collapsed + main sessions listed individually

**Source:** [commands.md](../../commands.md)

---

### IT-20: agent sessions shown individually at v2+

**Goal:** Verify that at verbosity 2, agent sessions are shown individually (no collapse), with entry counts.
**Setup:** Same as IT-19 (2 main + 3 agent sessions in one project).
**Command:** `clg .sessions scope::global verbosity::2`
**Verification:**
- Exit code is 0
- stdout DOES contain `agent-task-001`
- stdout does NOT contain `+ 3 agent` collapse line
- stdout contains entry counts per session
**Pass Criteria:** exit 0 + all 5 sessions listed individually + no collapse line

**Source:** [commands.md](../../commands.md)

---

### IT-21: entry count shown per session at v2+

**Goal:** Verify that `verbosity::2` appends `({n} entries)` to each session line.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with one project and one session containing exactly 4 entries.
**Command:** `clg .sessions scope::global verbosity::2`
**Expected Output:**
```
Found 1 session:

~/path/to/project:
  - session-id  (4 entries)
```
**Verification:**
- Exit code is 0
- stdout contains `(4 entries)` string
**Pass Criteria:** exit 0 + `(4 entries)` present in output

**Source:** [commands.md](../../commands.md)

---

### IT-22: agent::1 explicit filter disables collapse at v1

**Goal:** Verify that when `agent::1` is specified at verbosity 1, agent sessions are shown individually (no collapse), because the user explicitly requested agent sessions.
**Setup:** Same as IT-19 (2 main + 3 agent sessions in one project).
**Command:** `clg .sessions scope::global verbosity::1 agent::1`
**Verification:**
- Exit code is 0
- stdout DOES contain individual agent session IDs (`agent-task-001` etc.)
- stdout does NOT contain `+ 3 agent` collapse line
**Pass Criteria:** exit 0 + agent sessions listed individually when agent::1 set

**Source:** [commands.md](../../commands.md)

---

### IT-23: scope::under displays underscore dirs without splitting at `/`

**Goal:** Verify that `scope::under` project path headers display underscore-containing directory names correctly (e.g., `wip_core`) rather than splitting them on `/` (e.g., `wip/core`). Regression for issue-029: `decode_project_display` heuristic defaulted to `/` for all `-` boundaries, so encoded `wip-core` decoded to `wip/core` instead of `wip_core`.
**Setup:** Create real filesystem directories `/tmp/{tempdir}/wip_core/myproject/` so the FS-guided decoder can verify the correct path. `export CLAUDE_STORAGE_ROOT` pointing to a fixture root with a session in the path-encoded `wip_core/myproject` project.
**Command:** `clg .sessions scope::under path::/tmp/{tempdir}/wip_core verbosity::1`
**Expected Output:** stdout contains a line with `wip_core` in the project path header; no line contains `wip/core`.
**Verification:**
- Exit code is 0
- stdout contains `wip_core` (underscore preserved)
- No line in stdout contains `wip/core` (separator not incorrectly injected)
**Pass Criteria:** exit 0 + `wip_core` present in header + `wip/core` absent

**Source:** [commands.md](../../commands.md)

---

### IT-24: scope::global displays hyphen-prefixed topic dir in path header

**Goal:** Verify that the session path header includes a hyphen-prefixed topic directory (e.g., `src/-default_topic`) when that directory actually exists on disk. Regression for issue-030: `decode_project_display` stripped `--topic` suffixes before decoding, so a project stored under `src/-default_topic` displayed as `src` even when the topic directory was real.
**Setup:** Create real filesystem directory `{tempdir}/src/-default_topic/`. Write a session for the project at that path. `export CLAUDE_STORAGE_ROOT` and `HOME` to the temp dir.
**Command:** `clg .sessions scope::global verbosity::1`
**Expected Output:** stdout path header contains `-default_topic`; no line ends with `src:` (truncated form absent).
**Verification:**
- Exit code is 0
- stdout contains `-default_topic` in the project path header
- No line in stdout ends with `src:` (old truncated display absent)
- Session ID `session-topic-dir-test` appears in output
**Pass Criteria:** exit 0 + `-default_topic` in header + `src:` truncation absent

**Source:** [commands.md](../../commands.md)

---

### IT-25: scope::under excludes sibling with underscore-suffix name

**Goal:** Verify that `scope::under` with base `{tmp}/base` does NOT return sessions from the sibling directory `{tmp}/base_extra`, even though both encode to the same string prefix. Regression for issue-031: the string `starts_with` predicate matched `base_extra` (encoded `base-extra`) against the `base-` prefix, incorrectly including sibling sessions.
**Setup:** Create real filesystem directories `{tempdir}/base/sub/` (child) and `{tempdir}/base_extra/` (sibling). Write session `session-it25-child` for the child and `session-it25-sibling` for the sibling. `export CLAUDE_STORAGE_ROOT` and `HOME` to the temp dir.
**Command:** `clg .sessions scope::under path::{tempdir}/base`
**Expected Output:** stdout contains `session-it25-child`; stdout does NOT contain `session-it25-sibling`.
**Verification:**
- Exit code is 0
- `session-it25-child` appears (child `base/sub` is under `base`)
- `session-it25-sibling` is absent (sibling `base_extra` is NOT under `base`)
**Pass Criteria:** exit 0 + child session present + sibling session absent

**Source:** [commands.md](../../commands.md)

---

### IT-26: scope::relevant excludes sibling with underscore-suffix name

**Goal:** Verify that `scope::relevant` from a cwd of `{tempdir}/base_extra` does NOT include sessions from the sibling project `{tempdir}/base`, even though `base` is a string prefix of `base_extra` in encoded form. Regression for issue-032: `is_relevant_encoded` used string prefix matching, so `/base` falsely matched as an ancestor when base path was `/base_extra`.
**Setup:** Create real filesystem directories `{tempdir}/base/` (sibling) and `{tempdir}/base_extra/` (cwd). Write session `session-it26-sibling` for `base` and `session-it26-current` for `base_extra`. `export CLAUDE_STORAGE_ROOT` and `HOME` to the temp dir.
**Command:** `clg .sessions scope::relevant path::{tempdir}/base_extra`
**Expected Output:** stdout contains `session-it26-current`; stdout does NOT contain `session-it26-sibling`.
**Verification:**
- Exit code is 0
- `session-it26-current` appears (current project `base_extra` is always included under `relevant`)
- `session-it26-sibling` is absent (`base` is NOT an ancestor of `base_extra`)
**Pass Criteria:** exit 0 + current session present + sibling session absent

**Source:** [commands.md](../../commands.md)

---

### IT-27: entry count shown per session at v1

**Goal:** Verify that `verbosity::1` shows `({n} entries)` per session line (not only at v2+).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with one project and one session containing exactly 4 entries.
**Command:** `clg .sessions scope::global verbosity::1`
**Expected Output:**
```
Found 1 session:

/path/to/project: (1 session)
  * session-id  Xs ago  (4 entries)
```
**Verification:**
- Exit code is 0
- stdout contains `(4 entries)`
**Pass Criteria:** exit 0 + `(4 entries)` present at v1

**Source:** [commands.md](../../commands.md)

---

### IT-28: limit::N truncates main sessions shown at v1

**Goal:** Verify that `limit::2` with 5 main sessions shows only 2 and emits a `... and 3 more` truncation hint.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with one project containing 5 main sessions.
**Command:** `clg .sessions scope::global verbosity::1 limit::2`
**Verification:**
- Exit code is 0
- stdout contains `and 3 more` truncation hint
- At most 2 main session lines appear before the truncation hint
**Pass Criteria:** exit 0 + truncation hint present with correct count

**Source:** [commands.md](../../commands.md)

---

### IT-29: zero-byte sessions excluded from v1 display

**Goal:** Verify that a zero-byte JSONL placeholder file (B8 behaviour — Claude Code creates empty files on startup) is excluded from `verbosity::1` display and only real sessions appear.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with one project containing one real session (`session-real`, 2 entries) and one zero-byte file (`session-placeholder.jsonl`).
**Command:** `clg .sessions scope::global verbosity::1`
**Verification:**
- Exit code is 0
- `session-real` appears in stdout
- `session-placeholder` does NOT appear in stdout
**Pass Criteria:** exit 0 + real session present + zero-byte placeholder absent

**Source:** [commands.md](../../commands.md)

---

### IT-30: Summary header format (id, age, count, path)

**Goal:** Verify that the summary header line contains the session UUID truncated to 8 chars, a human-readable age string, the entry count, and the project path relative to cwd on the following line.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing one session with a known UUID and a known number of entries. Run from the project directory.
**Command:** `clg .sessions`
**Expected Output:**
```
Active session  {first-8-chars-of-uuid}  Xd ago  N entries
Project  {rel-path-from-cwd}
```
**Verification:**
- Exit code is 0
- stdout first line starts with `Active session`
- First line contains the first 8 chars of the session UUID
- First line contains the entry count followed by `entries`
- Second line starts with `Project` and contains the project path
**Pass Criteria:** exit 0 + header fields present: 8-char UUID, age, entry count, project path

**Source:** [commands.md](../../commands.md)

---

### IT-31: Truncation gate — message ≤ 50 chars shown in full

**Goal:** Verify that a last message of 50 characters or fewer is shown in full with no ellipsis.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing a session whose last text entry is exactly 40 characters (e.g. `Fix typo in the readme file near line 10`). Run from that project.
**Command:** `clg .sessions`
**Expected Output:** The `Last message:` section shows the full 40-char string; no `...` appears in the output.
**Verification:**
- Exit code is 0
- stdout contains the full 40-char message text
- stdout does NOT contain `...` in the last-message section
**Pass Criteria:** exit 0 + full message shown + no ellipsis

**Source:** [commands.md](../../commands.md)

---

### IT-32: Truncation formula — message > 50 chars as first30...last30

**Goal:** Verify that a last message longer than 50 characters is truncated to `{first30}...{last30}` (exactly 63 output characters), and the full message does not appear verbatim.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing a session whose last text entry is exactly 60 characters, with distinct known first-30 and last-30 substrings. Run from that project.
**Command:** `clg .sessions`
**Expected Output:** The `Last message:` section shows `{first30}...{last30}`. The full 60-char source text does NOT appear verbatim.
**Verification:**
- Exit code is 0
- stdout contains `...` in the last-message section
- The substring before `...` matches the first 30 chars of the source message
- The substring after `...` matches the last 30 chars of the source message
**Pass Criteria:** exit 0 + `...` present + first30 and last30 substrings match fixture

**Source:** [commands.md](../../commands.md)

---

### IT-33: No sessions in scope shows "No active session found."

**Goal:** Verify that when no sessions exist in scope, stdout contains `No active session found.` rather than an error or empty output.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (empty storage — no session files). Run from any directory.
**Command:** `clg .sessions`
**Expected Output:** `No active session found.`
**Verification:**
- Exit code is 0
- stdout contains `No active session found.`
- stderr is empty
- stdout does NOT contain `Active session`
**Pass Criteria:** exit 0 + `No active session found.` in stdout

**Source:** [commands.md](../../commands.md)

---

### IT-34: Explicit scope::local keeps list mode

**Goal:** Verify that providing any explicit parameter (`scope::local` here) bypasses summary mode and activates the normal session list.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing at least one session. Run from that project.
**Command:** `clg .sessions scope::local`
**Expected Output:** stdout contains `Found N session` (list-mode header); no `Active session` line.
**Verification:**
- Exit code is 0
- stdout contains `Found` followed by a session count (list-mode header)
- stdout does NOT contain `Active session` (summary mode not triggered)
**Pass Criteria:** exit 0 + `Found N session` header present + `Active session` absent

**Source:** [commands.md](../../commands.md)

---

### IT-35: Explicit limit::N keeps list mode

**Goal:** Verify that providing `limit::N` (an explicit parameter) bypasses summary mode and activates the normal session list.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing at least one session. Run from that project.
**Command:** `clg .sessions limit::5`
**Expected Output:** stdout contains `Found N session` (list-mode header); no `Active session` line.
**Verification:**
- Exit code is 0
- stdout contains `Found` followed by a session count (list-mode header)
- stdout does NOT contain `Active session` (summary mode not triggered)
**Pass Criteria:** exit 0 + `Found N session` header present + `Active session` absent

**Source:** [commands.md](../../commands.md)

---

### IT-36: Family header format (conversations + agents)

**Goal:** Verify that when a project has root sessions AND agent sessions, the project header at v1 shows `(N conversations, M agents)` instead of `(N sessions)`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project containing 1 root session and 3 agent sessions in hierarchical layout (`{uuid}/subagents/`).
**Command:** `clg .sessions scope::local`
**Expected Output:** Header contains `conversations` and `agents`.
**Verification:**
- Exit code is 0
- stdout contains `conversations`
- stdout contains `agents`
- stdout does NOT contain the old `+ ` agent collapse line
**Pass Criteria:** exit 0 + family header format + no legacy collapse

**Source:** [commands.md](../../commands.md)

---

### IT-37: Per-root agent breakdown [N agents: type summary]

**Goal:** Verify that each root session line at v1 includes an inline `[N agents: N×Type, …]` suffix showing the agent count and type distribution for that family.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project containing 1 root and 3 agents (2×Explore, 1×general-purpose) in hierarchical layout with meta.json sidecars.
**Command:** `clg .sessions scope::local`
**Expected Output:** Root session line contains `[3 agents: 2×Explore, 1×general-purpose]`.
**Verification:**
- Exit code is 0
- stdout contains `[3 agents:`
- stdout contains `Explore`
- stdout contains `general-purpose`
**Pass Criteria:** exit 0 + bracket breakdown present with correct counts and types

**Source:** [commands.md](../../commands.md)

---

### IT-38: Hierarchical format detection (subagents/ path)

**Goal:** Verify that agents stored in `{uuid}/subagents/` are correctly attributed to the root session whose UUID matches the directory name.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with 2 root sessions, each with distinct agents in their own `{uuid}/subagents/` directory.
**Command:** `clg .sessions scope::local`
**Expected Output:** Each root line shows only its own agent count, not the total.
**Verification:**
- Exit code is 0
- Each root session line has a distinct `[N agents:` count matching its agent set
**Pass Criteria:** exit 0 + agents attributed to correct parent

**Source:** [commands.md](../../commands.md)

---

### IT-39: Flat format detection (sessionId linkage)

**Goal:** Verify that flat-format agents (`agent-*.jsonl` at project root) are grouped by their `sessionId` field to the correct parent session.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with 1 root session and 2 flat agent files. Each agent's first JSONL entry has `"sessionId"` matching the root UUID.
**Command:** `clg .sessions scope::local`
**Expected Output:** Root line shows `[2 agents:` breakdown.
**Verification:**
- Exit code is 0
- Root session line contains `[2 agents:`
**Pass Criteria:** exit 0 + flat agents attributed to parent via sessionId

**Source:** [commands.md](../../commands.md)

---

### IT-40: Orphan family display (root missing)

**Goal:** Verify that agent sessions whose parent root `.jsonl` is missing are displayed as an orphan family with a `?` marker.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with `{uuid}/subagents/agent-*.jsonl` but NO `{uuid}.jsonl` root file.
**Command:** `clg .sessions scope::local`
**Expected Output:** Output contains `?` marker on the orphan line.
**Verification:**
- Exit code is 0
- stdout contains `?`
**Pass Criteria:** exit 0 + orphan marker present

**Source:** [commands.md](../../commands.md)

---

### IT-41: Childless root (no bracket suffix)

**Goal:** Verify that a root session with no agent sub-sessions does NOT display a `[` bracket suffix on its v1 line.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with 1 root session and 0 agents.
**Command:** `clg .sessions scope::local`
**Expected Output:** Root line has mtime and entry count but no `[` character.
**Verification:**
- Exit code is 0
- The root session line does NOT contain `[`
**Pass Criteria:** exit 0 + no bracket on childless root

**Source:** [commands.md](../../commands.md)

---

### IT-42: Meta.json agentType in breakdown

**Goal:** Verify that the agent type from `meta.json` appears in the family breakdown string.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with 1 root and 1 agent in hierarchical layout. The agent's `meta.json` contains `{"agentType":"Plan"}`.
**Command:** `clg .sessions scope::local`
**Expected Output:** Root line contains `Plan` in the bracket breakdown.
**Verification:**
- Exit code is 0
- stdout contains `Plan`
**Pass Criteria:** exit 0 + meta.json agentType shown in breakdown

**Source:** [commands.md](../../commands.md)

---

### IT-43: Empty/malformed meta.json fallback to "unknown"

**Goal:** Verify that when `meta.json` is empty (0 bytes), the agent type falls back to "unknown" in the breakdown.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with 1 root and 1 agent in hierarchical layout. The agent's `meta.json` file exists but is empty (0 bytes).
**Command:** `clg .sessions scope::local`
**Expected Output:** Root line contains `unknown` in the bracket breakdown.
**Verification:**
- Exit code is 0
- stdout contains `unknown`
**Pass Criteria:** exit 0 + "unknown" type in breakdown for empty meta.json

**Source:** [commands.md](../../commands.md)

### IT-44: v1 orphan shows `? (orphan)` label (bug-cc-c1)

**Goal:** Verify that at v1, an orphan family line shows `? (orphan)  [N agents: ...]` — including the `(orphan)` label — matching the spec in `commands.md`.
**Setup:** 1 flat agent session whose `sessionId` points to a non-existent root.
**Command:** `clg .sessions scope::local verbosity::1`
**Expected Output:** stdout contains `? (orphan)`
**Verification:**
- Exit code is 0
- stdout contains `? (orphan)` (label present, not just bare `?`)
**Pass Criteria:** exit 0 + `? (orphan)` present in output

**Source:** [commands.md](../../commands.md)

### IT-45: v2 root entry count singular — `(1 entry)` not `(1 entries)`

**Goal:** Verify that at v2+, a root session with exactly 1 entry shows `(1 entry)` not `(1 entries)`.
**Setup:** 1 root session with 1 JSONL entry.
**Command:** `clg .sessions scope::local verbosity::2`
**Expected Output:** stdout contains `(1 entry)` and does NOT contain `(1 entries)`
**Verification:**
- Exit code is 0
- stdout contains `(1 entry)`
- stdout does NOT contain `(1 entries)`
**Pass Criteria:** exit 0 + correct singular noun

**Source:** [commands.md](../../commands.md)

### IT-46: v2 agent entry count singular — `1 entry` not `1 entries`

**Goal:** Verify that at v2+, an agent with exactly 1 entry shows `1 entry` not `1 entries` on its tree-indented line.
**Setup:** 1 root + 1 hierarchical agent, each with 1 JSONL entry.
**Command:** `clg .sessions scope::local verbosity::2`
**Expected Output:** stdout contains `1 entry` and does NOT contain `1 entries`
**Verification:**
- Exit code is 0
- stdout contains `1 entry`
- stdout does NOT contain `1 entries`
**Pass Criteria:** exit 0 + correct singular noun for agent entry count

**Source:** [commands.md](../../commands.md)

---

### IT-47: verbosity::1 alone stays in summary mode (bug-is-default-verbosity)

**Goal:** Verify that passing `verbosity::1` (the default verbosity value) without any other parameter does NOT activate list mode — the output must be identical to bare `.sessions`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing at least one session with entries. Run from that project.
**Command:** `clg .sessions verbosity::1`
**Expected Output:** Same summary block as bare `clg .sessions` — NOT a session list.
```
Active session  {8-char-id}  Xd ago  N entries
Project  ~/path/to/project

Last message:
  {message text}
```
stdout does NOT contain `Found N sessions:` (list-mode header must be absent).
**Verification:**
- Exit code is 0
- stdout first line contains `Active session`
- stdout contains `Project` line
- stdout contains `Last message:` header
- stdout does NOT contain `Found N sessions:` (list mode must not activate)
**Pass Criteria:** exit 0 + summary header present + `Found N sessions:` absent

**Root Cause (bug-is-default-verbosity):** `is_default` gate in `sessions_routine` included `verbosity` in its all-None check (`cmd.get_integer("verbosity").is_none()`). Passing `verbosity::1` returned `Some(1)` instead of `None`, setting `is_default=false` and routing to list mode even though `verbosity::1` is semantically equivalent to the default.

**Source:** [commands.md](../../commands.md)
