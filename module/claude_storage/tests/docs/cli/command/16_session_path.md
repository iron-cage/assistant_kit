# Command :: `.session.path`

Integration tests for the `.session.path` command, implemented in `tests/cli_cmd_session_path_test.rs`. Tests verify the three mutually exclusive selectors (`latest::`, `session::`, `topic::`), the disk-reading `latest` default and its empty-storage exit, the pure-computation `session::` join, the fork-mode `UUIDv5` sense of `topic::` that deliberately diverges from every other command's `-{topic}` dir sense, selector validation, and a published golden vector pinning the cross-implementation `UUIDv5` contract end-to-end through the binary.

**Source:** [command/16_session_path.md](../../../../docs/cli/command/16_session_path.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| SP-1 | Default selector resolves the storage's most recent session file | Default Resolution |
| SP-2 | `latest::1` explicit is byte-identical to the default selector | Default Resolution |
| SP-3 | Latest against empty storage exits 2 with `no sessions` on stderr | Exit Codes |
| SP-4 | Latest picks the newer of two sessions by mtime | Default Resolution |
| SP-5 | `session::UUID` is pure computation — no existence check | Selector Semantics |
| SP-6 | `topic::NAME` resolves via the fork-mode `UUIDv5` rule | Selector Semantics |
| SP-7 | `session::` / `latest::` / `topic::` are mutually exclusive | Input Validation |
| SP-8 | Empty or slash-containing `topic::` / `session::` rejected | Input Validation |
| SP-9 | Golden vector — `path::/tmp/x topic::a` yields the published `UUIDv5` filename | Contract Pinning |

## Test Coverage Summary

- Default Resolution: 3 tests (SP-1, SP-2, SP-4)
- Selector Semantics: 2 tests (SP-5, SP-6)
- Input Validation: 2 tests (SP-7, SP-8)
- Exit Codes: 1 test (SP-3)
- Contract Pinning: 1 test (SP-9)

**Total:** 9 integration cases

**Behavioral Divergence Pair:** SP-5 (`session::`, never touches disk) ↔ SP-1/SP-4 (`latest`, the only selector that reads it)

## Topic Sense Collision (deliberate)

Every other `claude_storage` command reads `topic::` as the legacy dir-suffix sense `{base}/-{topic}`. On `.session.path` it instead names a fork-mode topic: the value selects the deterministic file `{storage}/{UUIDv5( canonical base, name )}.jsonl` inside the BASE directory's own storage. SP-6 pins the fork sense and asserts the absence of a `/-alpha` component, so a regression back to the dir sense fails loudly rather than silently resolving elsewhere. See [`command/16_session_path.md`](../../../../docs/cli/command/16_session_path.md) § Topic Sense Collision and [`param/17_topic.md`](../../../../docs/cli/param/17_topic.md).

## Test Cases

---

### SP-1: Default selector resolves the storage's most recent session file

**Command:**
```
HOME=/tmp/isolated-home clg .session.path path::PROJECT
```

**Expected behavior:**
- Fixture: one project with a single 2-entry session `11111111-…-111111111111`
- Stdout is exactly that session's absolute `.jsonl` path plus a trailing newline
- Path is absolute (starts with `/`)
- Exit code: 0
- **Source:** [command/16_session_path.md](../../../../docs/cli/command/16_session_path.md)

---

### SP-2: `latest::1` explicit is byte-identical to the default selector

**Command:**
```
HOME=/tmp/isolated-home clg .session.path path::PROJECT latest::1
```

**Expected behavior:**
- Same fixture as SP-1
- Stdout byte-identical to the bare invocation — `latest::1` names the effective default rather than switching behavior
- Exit code: 0
- **Source:** [param/41_latest.md](../../../../docs/cli/param/41_latest.md)

---

### SP-3: Latest against empty storage exits 2 with `no sessions` on stderr

**Command:**
```
HOME=/tmp/isolated-home clg .session.path path::EMPTY_PROJECT
```

**Expected behavior:**
- Fixture: a project directory with no storage sessions written
- Stderr contains `no sessions`; stdout is empty (nothing partial is printed before the failure)
- Exit code: 2
- **Source:** [command/16_session_path.md](../../../../docs/cli/command/16_session_path.md)

---

### SP-4: Latest picks the newer of two sessions by mtime

**Command:**
```
HOME=/tmp/isolated-home clg .session.path path::PROJECT
```

**Expected behavior:**
- Fixture: two sessions written in sequence with a sleep between them so the second is strictly newer
- Stdout names the second-written session, not the first
- Exit code: 0
- **Source:** [param/41_latest.md](../../../../docs/cli/param/41_latest.md)

---

### SP-5: `session::UUID` is pure computation — no existence check

**Command:**
```
HOME=/tmp/isolated-home clg .session.path path::PROJECT session::55555555-5555-5555-5555-555555555555
```

**Expected behavior:**
- Fixture: project exists but that session was never written, and storage may be empty
- Stdout is `{storage}/55555555-….jsonl` — a pure join, succeeding despite the file's absence
- Exit code: 0
- **Source:** [param/13_session.md](../../../../docs/cli/param/13_session.md)

---

### SP-6: `topic::NAME` resolves via the fork-mode `UUIDv5` rule

**Command:**
```
HOME=/tmp/isolated-home clg .session.path path::PROJECT topic::alpha
```

**Expected behavior:**
- Stdout is `{base storage}/{UUIDv5( canonical base, "alpha" )}.jsonl`, matching `claude_storage_core::topic_session_id()` for the same inputs
- Output contains no `/-alpha` component — the legacy dir-suffix sense must not appear
- Exit code: 0
- **Source:** [param/17_topic.md](../../../../docs/cli/param/17_topic.md)

---

### SP-7: `session::` / `latest::` / `topic::` are mutually exclusive

**Command:**
```
HOME=/tmp/isolated-home clg .session.path path::PROJECT session::66666666-… latest::1
HOME=/tmp/isolated-home clg .session.path path::PROJECT topic::alpha latest::1
HOME=/tmp/isolated-home clg .session.path path::PROJECT session::66666666-… topic::alpha
```

**Expected behavior:**
- Each of the three pairs is rejected; combined stdout+stderr mentions `mutually exclusive`
- All three pairs are checked, not just one — an implementation guarding only the pair it was written against still fails here
- Exit code: 1
- **Source:** [command/16_session_path.md](../../../../docs/cli/command/16_session_path.md)

---

### SP-8: Empty or slash-containing `topic::` / `session::` rejected

**Command:**
```
HOME=/tmp/isolated-home clg .session.path path::PROJECT topic::
HOME=/tmp/isolated-home clg .session.path path::PROJECT topic::sub/dir
HOME=/tmp/isolated-home clg .session.path path::PROJECT session::
HOME=/tmp/isolated-home clg .session.path path::PROJECT session::a/b
```

**Expected behavior:**
- Each of the four values produces non-empty error output — a slash would otherwise escape the storage directory through the path join
- Exit code: 1 for all four
- **Source:** [param/17_topic.md](../../../../docs/cli/param/17_topic.md)

---

### SP-9: Golden vector — `path::/tmp/x topic::a` yields the published `UUIDv5` filename

**Command:**
```
HOME=/tmp/isolated-home clg .session.path path::/tmp/x topic::a
```

**Expected behavior:**
- Stdout ends with `/-tmp-x/41299c24-a8f5-589f-9fce-8474fc855532.jsonl`
- Pins the cross-implementation contract (namespace plus NUL-separated name layout) end-to-end through the binary, matching the core golden-vector unit tests — a change to either the namespace UUID or the name encoding breaks this case even when every other case still passes
- Exit code: 0
- **Source:** [command/16_session_path.md](../../../../docs/cli/command/16_session_path.md)
