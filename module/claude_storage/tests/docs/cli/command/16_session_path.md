# Command :: `.session.path`

Integration tests for the `.session.path` command. Tests verify session file path resolution for all three selectors (`latest::` default, `session::`, fork-mode `topic::`), base-path canonicalization, selector mutual exclusion, validation, and the exit-2 empty-storage contract.

**Source:** [command/16_session_path.md](../../../../docs/cli/command/16_session_path.md)
**Implementation:** `tests/cli_cmd_session_path_test.rs` (fn prefix `sp_`)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| SP-1 | Default selector = latest — resolves most recent session file | Basic Behavior |
| SP-2 | `latest::1` explicit — byte-identical to default | Basic Behavior |
| SP-3 | Latest with empty storage → exit 2 + "no sessions" | Exit Codes |
| SP-4 | Latest picks the newer of two sessions (mtime ordering) | Selection Ordering |
| SP-5 | `session::UUID` is pure join — succeeds with no storage on disk | Selector Semantics |
| SP-6 | `topic::NAME` resolves via fork-mode UUIDv5 rule, not `-{topic}` dir | Selector Semantics |
| SP-7 | `session::`/`latest::`/`topic::` mutually exclusive | Validation |
| SP-8 | Empty or slash-containing `topic::`/`session::` rejected | Validation |
| SP-9 | Golden vector: `path::/tmp/x topic::a` → `41299c24-…-8474fc855532.jsonl` | Contract Pinning |

## Test Coverage Summary

- Basic Behavior: 2 tests (SP-1, SP-2)
- Exit Codes: 1 test (SP-3)
- Selection Ordering: 1 test (SP-4)
- Selector Semantics: 2 tests (SP-5, SP-6)
- Validation: 2 tests (SP-7, SP-8)
- Contract Pinning: 1 test (SP-9)

## Test Cases

---

### SP-1: Default selector = latest — resolves most recent session file

**Command:**
```
clg .session.path path::{tempdir}
```

**Expected behavior:**
- Fixture: one session file written under the CANONICALIZED tempdir's storage; the RAW (possibly symlinked) tempdir path is passed as `path::` — proving the command canonicalizes before encoding
- Output: absolute path `{home}/.claude/projects/{encode_path( canonical base )}/{id}.jsonl`, single line
- Exit code: 0
- **Source:** [command/16_session_path.md](../../../../docs/cli/command/16_session_path.md), [param/09_path.md](../../../../docs/cli/param/09_path.md)

---

### SP-2: `latest::1` explicit — byte-identical to default

**Command:**
```
clg .session.path path::{tempdir} latest::1
```

**Expected behavior:**
- Identical stdout to the selector-less invocation of SP-1 against the same fixture
- Exit code: 0
- **Source:** [command/16_session_path.md](../../../../docs/cli/command/16_session_path.md), [param/41_latest.md](../../../../docs/cli/param/41_latest.md)

---

### SP-3: Latest with empty storage → exit 2 + "no sessions"

**Command:**
```
clg .session.path path::{tempdir}
```

**Expected behavior:**
- Fixture: no storage directory exists for the base
- stderr contains `no sessions in {storage}`
- Exit code: 2 (distinguishes "nothing to resolve" from usage errors, mirroring the `.status` exit-2 precedent)
- **Source:** [command/16_session_path.md](../../../../docs/cli/command/16_session_path.md), [param/41_latest.md](../../../../docs/cli/param/41_latest.md)

---

### SP-4: Latest picks the newer of two sessions (mtime ordering)

**Command:**
```
clg .session.path path::{tempdir}
```

**Expected behavior:**
- Fixture: two session files written ~1.1s apart (mtime granularity margin)
- Output ends with the second (newer) session's filename
- Exit code: 0
- **Source:** [command/16_session_path.md](../../../../docs/cli/command/16_session_path.md), [param/41_latest.md](../../../../docs/cli/param/41_latest.md)

---

### SP-5: `session::UUID` is pure join — succeeds with no storage on disk

**Command:**
```
clg .session.path path::{tempdir} session::11111111-2222-3333-4444-555555555555
```

**Expected behavior:**
- Fixture: storage directory deliberately absent — no existence check is performed
- Output: `{storage}/11111111-2222-3333-4444-555555555555.jsonl`
- Exit code: 0
- **Source:** [command/16_session_path.md](../../../../docs/cli/command/16_session_path.md), [param/13_session.md](../../../../docs/cli/param/13_session.md)

---

### SP-6: `topic::NAME` resolves via fork-mode UUIDv5 rule, not `-{topic}` dir

**Command:**
```
clg .session.path path::{tempdir} topic::alpha
```

**Expected behavior:**
- Output: `{storage of BASE}/{UUIDv5( canonical base NUL "alpha" )}.jsonl` — the expected UUID computed in-test via `claude_storage_core::topic_session_id`
- Output does NOT contain `/-alpha` (the legacy dir-suffix sense used by every other `topic::` consumer)
- Exit code: 0
- Cross-binary parity: `clr topics --file alpha` pins its output to the same core-computed value (claude_runner test F15), so the two binaries agree byte-for-byte by transitivity
- **Source:** [command/16_session_path.md](../../../../docs/cli/command/16_session_path.md), [param/17_topic.md](../../../../docs/cli/param/17_topic.md)

---

### SP-7: `session::`/`latest::`/`topic::` mutually exclusive

**Command:**
```
clg .session.path path::{tempdir} session::{uuid} latest::1
clg .session.path path::{tempdir} session::{uuid} topic::x
clg .session.path path::{tempdir} latest::1 topic::x
```

**Expected behavior:**
- All three pairings rejected; stderr contains `mutually exclusive`
- Exit code: 1
- **Source:** [command/16_session_path.md](../../../../docs/cli/command/16_session_path.md), [param/41_latest.md](../../../../docs/cli/param/41_latest.md)

---

### SP-8: Empty or slash-containing `topic::`/`session::` rejected

**Command:**
```
clg .session.path path::{tempdir} topic::
clg .session.path path::{tempdir} topic::a/b
clg .session.path path::{tempdir} session::
clg .session.path path::{tempdir} session::a/b
```

**Expected behavior:**
- All four rejected with a validation error on stderr
- Exit code: 1
- **Source:** [command/16_session_path.md](../../../../docs/cli/command/16_session_path.md), [param/13_session.md](../../../../docs/cli/param/13_session.md), [param/17_topic.md](../../../../docs/cli/param/17_topic.md)

---

### SP-9: Golden vector: `path::/tmp/x topic::a` → published UUIDv5 filename

**Command:**
```
clg .session.path path::/tmp/x topic::a
```

**Expected behavior:**
- Fixture: `/tmp/x` created if absent (canonicalization requires an existing path)
- Output ends with `/-tmp-x/41299c24-a8f5-589f-9fce-8474fc855532.jsonl` — the published golden vector for namespace `f2b5cc6a-c186-5cc7-99db-3075d9c705f8` with name layout `{canonical base}\0{topic}`
- Exit code: 0
- Pins the cross-implementation contract end-to-end through the CLI, matching the core golden-vector unit tests
- **Source:** [command/16_session_path.md](../../../../docs/cli/command/16_session_path.md), [param/17_topic.md](../../../../docs/cli/param/17_topic.md)
