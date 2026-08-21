# Command :: `.tail`

Integration tests for the `.tail` command, in two families.

**Resolution** (INT-1..INT-11) verifies zero-parameter defaults, window size control, topic resolution, the most-recently-modified-session fallback, and not-found handling. Its fixtures are built from entry counts; record shape is irrelevant, and each fixture entry happens to form its own turn, so "entries" and "turns" coincide there.

**Rendering** (INT-12..INT-23) verifies how records become turns and how turns are drawn. Its fixtures are raw JSONL written line by line, because every case is *about* a record shape — a shared `message.id`, array-form content, a `tool_use`/`tool_result` pair, an unmodelled block type. These shapes cannot be expressed by an entry count.

**Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | No args, single session present, prints its last 4 entries | Location-Aware |
| INT-2 | last::N controls entry count | Entry Count |
| INT-3 | last::0 prints all entries | Entry Count |
| INT-4 | topic:: resolves a non-default session | Topic Resolution |
| INT-5 | path:: resolves a different directory's project | Project Scope |
| INT-6 | Fewer entries than requested prints all available | Boundary |
| INT-7 | Exit code 2 when cwd has no project | Exit Codes |
| INT-8 | Negative last:: is rejected with exit code 1 | Input Validation |
| INT-9 | No args falls back to the most recent session when no `-default_topic` session exists | Recency Fallback |
| INT-10 | No args picks the most recently modified session among multiple candidates | Recency Fallback |
| INT-11 | No args excludes agent sessions from the most-recent fallback | Recency Fallback |
| INT-12 | Consecutive records sharing one `message.id` collapse into one turn | Turn Grouping |
| INT-13 | Array-form user `message.content` is parsed, not silently dropped | Content Parsing |
| INT-14 | A tool call renders its input summary and its result's line count | Tool Rendering |
| INT-15 | A turn holding only `tool_result` blocks never consumes a `last::` slot | Turn Grouping |
| INT-16 | Empty text and thinking blocks render nothing, not a bare label | Turn Grouping |
| INT-17 | Turns past 8 body lines fold; `full::1` unfolds them | Layout |
| INT-18 | `compact::1` prints one line per turn | Layout |
| INT-18b | `compact::1 full::1` — compact wins | Layout |
| INT-19 | Session header reports project, session id, and turn span | Layout |
| INT-20 | Output ends with exactly one newline | Layout |
| INT-21 | An unmodelled block type is marked, not dropped with its record | Content Parsing |
| INT-22 | A failed tool call is annotated `↳ error` | Tool Rendering |
| INT-23 | Array-form `tool_result.content` flattens instead of rejecting the record | Content Parsing |
| INT-24 | A tool with no path/command key still summarises (`status` outranks `taskId`) | Tool Rendering |

## Test Coverage Summary

- Location-Aware: 1 test (INT-1)
- Entry Count: 2 tests (INT-2, INT-3)
- Topic Resolution: 1 test (INT-4)
- Project Scope: 1 test (INT-5)
- Boundary: 1 test (INT-6)
- Exit Codes: 1 test (INT-7)
- Input Validation: 1 test (INT-8)
- Recency Fallback: 3 tests (INT-9, INT-10, INT-11)
- Turn Grouping: 3 tests (INT-12, INT-15, INT-16)
- Content Parsing: 3 tests (INT-13, INT-21, INT-23)
- Tool Rendering: 3 tests (INT-14, INT-22, INT-24)
- Layout: 5 tests (INT-17, INT-18, INT-18b, INT-19, INT-20)

**Total:** 25 integration cases

## Test Cases

---

### INT-1: No args, single session present, prints its last 4 entries

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: a project whose path-encoding matches the test's cwd, with a `-default_topic` session containing 6 known entries
- The last 4 entries printed, oldest-first, as conversation content
- Exit code: 0
- The session is selected by the most-recently-modified-non-agent-session fallback (BUG-488); it is the only session in the fixture, so it is trivially both the default-named and the most recent one — this test does not by itself distinguish name-based resolution from recency-based resolution (see INT-9 for that distinction)
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-2: last::N controls entry count

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail last::2
```

**Expected behavior:**
- Fixture: same project, `-default_topic` session with 6 known entries
- Exactly the last 2 entries printed, oldest-first
- Exit code: 0
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-3: last::0 prints all entries

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail last::0
```

**Expected behavior:**
- Fixture: `-default_topic` session with 6 known entries
- All 6 entries printed, oldest-first
- Exit code: 0
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-4: topic:: resolves a non-default session

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail topic::work
```

**Expected behavior:**
- Fixture: project with both a `-default_topic` session and a `-work` session, each with distinct known content
- The last 4 entries from the `-work` session printed; no `-default_topic` content shown
- Exit code: 0
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-5: path:: resolves a different directory's project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail path::/home/alice/projects/alpha
```

**Expected behavior:**
- Fixture: project `alpha` with a `-default_topic` session; run from a cwd that does not correspond to `alpha`
- The last 4 entries from `alpha`'s `-default_topic` session printed
- Exit code: 0
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-6: Fewer entries than requested prints all available

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail last::10
```

**Expected behavior:**
- Fixture: `-default_topic` session with only 3 known entries
- All 3 entries printed, oldest-first; no error or padding
- Exit code: 0
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-7: Exit code 2 when cwd has no project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: run from a directory (e.g., `/tmp`) that has no matching storage project
- Error message on stderr indicating the current directory has no project in storage
- Exit code: 2
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-8: Negative last:: is rejected with exit code 1

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail last::-1
```

**Expected behavior:**
- Fixture: same project, `-default_topic` session with 6 known entries (rejection happens before entries are loaded)
- Error message on stderr: exactly `"last must be non-negative"`
- Exit code: 1
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md), [param/25_last.md](../../../../docs/cli/param/25_last.md)

---

### INT-9: No args falls back to the most recent session when no `-default_topic` session exists

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: a project with exactly one UUID-named session (no `-default_topic` session written at all), containing 6 known entries
- The last 4 entries printed, oldest-first, from the UUID-named session
- Exit code: 0
- Regression coverage for BUG-488: real Claude Code sessions are UUID-named, never topic-tagged, so the zero-parameter default must not require a literal `-default_topic` session to exist
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-10: No args picks the most recently modified session among multiple candidates

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: a project with two UUID-named sessions written with distinguishable mtimes (older written first, then a delay, then the newer)
- Output contains the newer session's marker content; the older session's content is not required to resolve the assertion
- Exit code: 0
- Confirms the fallback actually compares modification times rather than picking an arbitrary/first-found session
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-11: No args excludes agent sessions from the most-recent fallback

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: a project with one main (UUID-named) session and one `agent-*`-named sidecar session, the agent session written more recently (newer mtime) than the main session
- Output contains the main session's content; the agent session's content is absent
- Exit code: 0
- Confirms agent sidecar sessions are never selected by the recency fallback even when they are the newest file in the project, matching `claude_storage_core::continuation`'s own established agent-exclusion convention
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-12: Consecutive records sharing one `message.id` collapse into one turn

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail last::1
```

**Expected behavior:**
- Fixture: three raw assistant records — one under `msg_earlier`, then two consecutive records both under `msg_shared`, each carrying a distinct marker
- Both `msg_shared` fragments printed under a single rule line; `msg_earlier`'s marker absent
- Exactly one rule line drawn — `last::1` selects one *turn*, which is two records
- Exit code: 0
- Without grouping, `last::1` would return the trailing fragment of a response and silently discard the rest of the same answer
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-13: Array-form user `message.content` is parsed, not silently dropped

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: two raw user records — one with string `message.content`, one with array-form content holding a single `text` block
- Both markers printed
- Exit code: 0
- Regression coverage for the silent-skip failure mode: `load_entries` drops any line the parser rejects, so a parser that accepted only the string form made the array-form majority of user records invisible rather than erroring
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-14: A tool call renders its input summary and its result's line count

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: an assistant `tool_use` record (`Bash`, `command: "git status --short"`) followed by the user `tool_result` record answering it with a 3-line body
- Output contains `⚙ Bash · git status --short` and the right-aligned annotation `↳ 3 lines`
- The result body itself is not printed
- Exit code: 0
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-15: A turn holding only `tool_result` blocks never consumes a `last::` slot

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: four raw records — user question, assistant `tool_use`, user `tool_result`, assistant answer
- Exactly three rule lines drawn; the header reports `turns 1-3 of 3`; both the question and the answer markers appear
- Exit code: 0
- The `tool_result` record renders nothing of its own (its content is folded onto the `⚙` line), so counting it as a turn would silently shrink the visible window
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-16: Empty text and thinking blocks render nothing, not a bare label

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: one assistant record whose only block is empty `thinking`, then one whose blocks are an empty `text` and a non-empty `text`
- No `Thinking ·` label anywhere in the output; the non-empty sibling's marker present; exactly one rule line drawn — the empty-only turn is dropped entirely
- Exit code: 0
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-17: Turns past 8 body lines fold; `full::1` unfolds them

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail full::1
```

**Expected behavior:**
- Fixture: one assistant record with a 20-line body
- Default run: body lines 1-8 present, line 9 absent, and a `⋯ 12 more lines` hint naming a `.show session_id::… index::…` invocation that resolves to this turn
- `full::1` run: all 20 lines present, no `⋯` anywhere
- Exit code: 0 for both
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md), [param/42_full.md](../../../../docs/cli/param/42_full.md)

---

### INT-18: `compact::1` prints one line per turn

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail compact::1
```

**Expected behavior:**
- Fixture: three turns — user, assistant, user — each carrying a distinct marker
- Exactly three marker-bearing rows; zero rule lines; the first row carries ordinal `1` and speaker `You`, the second names `Claude`
- Exit code: 0
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md), [param/43_compact.md](../../../../docs/cli/param/43_compact.md)

---

### INT-18b: `compact::1 full::1` — compact wins

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail compact::1 full::1
```

**Expected behavior:**
- Fixture: a user turn plus an assistant turn with a 20-line body — long enough that `full::` demonstrably changes the *default* layout, so its inertness here is a real observation rather than a vacuous one
- Output byte-identical to `compact::1` alone over the same fixture (both runs share one fixture, so the project label in the header matches and the whole output can be compared)
- No unfolded body lines; zero rule lines
- Exit code: 0
- **Source:** [param/43_compact.md](../../../../docs/cli/param/43_compact.md), [param/42_full.md](../../../../docs/cli/param/42_full.md)

---

### INT-19: Session header reports project, session id, and turn span

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail last::2
```

**Expected behavior:**
- Fixture: five assistant records, each with its own `message.id`
- First output line carries the 8-character session-id prefix and the span `turns 4-5 of 5`
- Exit code: 0
- Without the span, a tail gives no indication of how much history sits above it
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-20: Output ends with exactly one newline

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: a single assistant record with a one-line body
- stdout ends with the body line followed by exactly one `\n` — no trailing blank line
- Exit code: 0
- The rendered string carries no trailing newline of its own: the CLI prints it through `println!`, which supplies the only one
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-21: An unmodelled block type is marked, not dropped with its record

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: one user record whose content array holds an `image` block followed by a `text` block
- Both the `⧉ image` marker and the text marker printed
- Exit code: 0
- Graceful degradation against schema drift: rejecting the block would reject the whole record, and a rejected record is invisible rather than loud
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-22: A failed tool call is annotated `↳ error`

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: an assistant `tool_use` record answered by a `tool_result` with `is_error: true`
- Output contains `↳ error` on the `⚙` line
- Exit code: 0
- Failure must be visible at the call site; folded to `↳ 2 lines` it would read as an ordinary result
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-23: Array-form `tool_result.content` flattens instead of rejecting the record

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: an assistant `tool_use` (`Read`, `file_path: "/tmp/x.rs"`) answered by a `tool_result` whose `content` is an array of two nested `text` blocks
- Output contains `⚙ Read · /tmp/x.rs` and `↳ 2 lines` — the nested blocks joined with a newline
- Exit code: 0
- The second of the two shapes Claude Code writes for `tool_result.content`; requiring a string rejected the whole record
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-24: A tool with no path/command key still summarises

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: one assistant `tool_use` record for `TaskUpdate`, whose input is `{"taskId": "42", "status": "completed"}` — no `command`, no `file_path`, no other originally-listed summary key
- Output contains `⚙ TaskUpdate · completed`; the opaque `42` does not appear as the summary
- Exit code: 0
- `TaskUpdate` is the most common tool in the local store after the file and shell tools, and every one of its calls rendered as a bare `⚙ TaskUpdate` before `status` was listed; the key order is what decides whether the line says what happened or shows an id
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)
