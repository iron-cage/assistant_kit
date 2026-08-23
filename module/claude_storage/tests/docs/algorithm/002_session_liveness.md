# Algorithm :: Session Liveness

Direct contract tests for the liveness inference documented in the session liveness algorithm.

**Source:** [algorithm/002_session_liveness.md](../../../docs/algorithm/002_session_liveness.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| AL-3 | Absent process table reports nothing rather than everything dead | Detection Availability |
| AL-4 | A `claude` process' cwd marks its project attached; other processes do not | Signal 1 — Attached Processes |
| AL-5 | An attached project splits into working and waiting by recency | Working/Waiting Split |
| AL-6 | An mtime ahead of the clock is working, not waiting | Working/Waiting Split |
| AL-7 | History pins the driven session over a newer sibling | Signal 2 — History Correlation |
| AL-8 | Absent history falls back to mtime rank, bounded by attached count | Signal 2 — History Correlation |
| AL-9 | Two attached processes cap the driven set at two, newest-first | Signal 2 — History Correlation |
| AL-10 | History for an unattached project is ignored entirely | Signal 2 — History Correlation |
| AL-11 | A valid history record survives malformed lines ahead of it | Signal 2 — History Correlation |
| AL-12 | No label exceeds the width the column reserves | Rendering Contract |

## Test Coverage Summary

- Detection Availability: 1 test (AL-3)
- Signal 1 — Attached Processes: 1 test (AL-4)
- Working/Waiting Split: 2 tests (AL-5, AL-6)
- Signal 2 — History Correlation: 5 tests (AL-7..AL-11)
- Rendering Contract: 1 test (AL-12)

**Total:** 10 algorithm contract cases

**Implementation target:** `tests/cli_liveness_unit_test.rs`

Each case builds a real `/proc`-shaped directory (numeric subdirectories holding
a `comm` file and a `cwd` symlink) and a real `history.jsonl` inside a `TempDir`,
then calls `LivenessMap::probe` against them — the same `read_dir`,
`read_to_string`, and `read_link` calls the live path makes against the kernel's
own filesystem. No signal is simulated.

## Test Cases

---

### AL-3: Absent process table reports nothing rather than everything dead

- **Given:** a `proc_dir` path that does not exist
- **When:** `LivenessMap::probe` runs against it
- **Then:** `any_attached()` is false and `project_state()` is `None` for every path — an unreadable process table must not claim knowledge, per the algorithm's "detection can only report positives" constraint

---

### AL-4: A `claude` process' cwd marks its project attached; other processes do not

- **Given:** a process table holding pid 101 with `comm` = `claude` and cwd at project A, and pid 102 with `comm` = `bash` and cwd at project B
- **When:** the map is probed
- **Then:** project A reports a state and project B reports `None` — matching `comm` exactly is what excludes wrapper scripts and the probing process itself

---

### AL-5: An attached project splits into working and waiting by recency

- **Given:** an attached project whose newest session write is within 60 s, and a second whose newest write is well outside it
- **When:** `project_state` is asked for each
- **Then:** the first is `Working` and the second is `Waiting` — and, crucially, the long-idle project stays live rather than decaying to `None`, which is the failure mode a recency cutoff would produce

---

### AL-6: An mtime ahead of the clock is working, not waiting

- **Given:** an attached project whose session mtime is in the future (clock skew against an NFS or container host, a restored archive, a deliberate `touch -d`)
- **When:** the working/waiting split is applied
- **Then:** the state is `Working` — `duration_since` signals a future timestamp as `Err`, and folding that error in with "too old" would rank the newest possible write least active

---

### AL-7: History pins the driven session over a newer sibling

- **Given:** an attached project with two sessions, where `history.jsonl` names the *older* one as driven
- **When:** `session_state` is asked for both
- **Then:** the history-named session is live and the newer sibling is `None` — history is authoritative whenever it has anything to say, precisely because the newest session by mtime is frequently not the live one

---

### AL-8: Absent history falls back to mtime rank, bounded by attached count

- **Given:** an attached project with no history record at all (a headless `--print` session, which takes its prompt on argv and never writes history)
- **When:** `session_state` is asked by mtime rank
- **Then:** ranks below the attached count are live and ranks at or above it are `None` — mtime order is the only signal left, and the attached count is what bounds it

---

### AL-9: Two attached processes cap the driven set at two, newest-first

- **Given:** one project with two attached `claude` processes and three history records naming three distinct sessions
- **When:** `session_state` is asked for each
- **Then:** the two newest records are live and the oldest is `None` — history is read newest-first and capped at the project's attached count, so the cap is two here rather than one or three

---

### AL-10: History for an unattached project is ignored entirely

- **Given:** a history record naming a session in a project with no attached process
- **When:** the map is probed
- **Then:** that project reports `None` — history is correlated only for projects already known to be attached, so a record alone never manufactures liveness

---

### AL-11: A valid history record survives malformed lines ahead of it

- **Given:** a `history.jsonl` holding `not json at all`, then `{"partial":true}`, then one valid record
- **When:** the tail is read
- **Then:** the valid record is honoured even at mtime rank 5 — a rank the mtime fallback alone would reject, so the assertion proves the record was parsed rather than the fallback firing. Neither a wholly unparseable line nor a structurally valid but incomplete one may abort the pass.

---

### AL-12: No label exceeds the width the column reserves

- **Given:** the `Working` and `Waiting` labels and `Liveness::column_width()`
- **When:** each label's character count is compared against the reserved width
- **Then:** neither exceeds it — the column is sized before its cells exist, so a label grown past the reserved width would misalign every row
