# Test: Feature — Journaling Integration

### Scope

- **Purpose**: FT- test cases verifying journal event emission, level control, and directory resolution for `clr` execution boundaries.
- **Responsibility**: Acceptance criteria confirming journal level semantics (full/meta/off), directory precedence, truncation, error isolation, and gate/retry/timeout event emission.
- **In Scope**: `--journal` levels, `--journal-dir`/`CLR_JOURNAL_DIR` resolution and precedence, stdout truncation at 1MB, write-failure isolation, gate/validation/retry/timeout event emission, flag validation errors.
- **Out of Scope**: default command assembly and dry-run gates (-> `001_runner_tool.md`), retry count/delay tier resolution (-> `003_retry_hierarchy.md`), JSON config loading (-> `004_json_config.md`).

Test case planning for [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md). Tests validate journaling emission at execution boundaries, level control (full/meta/off), directory resolution, truncation, and error isolation.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | `--journal full` (default) — event written with stdout field populated | Level Default |
| FT-2 | `--journal meta` — event written without stdout/stderr fields | Level Divergence |
| FT-3 | `--journal off` — no event file created in journal dir | Level Divergence |
| FT-4 | No `--journal` flag — journals at `full` level by default (AC-004) | Default Behavior |
| FT-5 | `--journal-dir /tmp/j` — event written to `/tmp/j/YYYY-MM-DD.jsonl` | Directory Override |
| FT-6 | `CLR_JOURNAL_DIR=/tmp/j` env var — same directory effect as CLI flag | Env Var |
| FT-7 | Journal write failure (read-only path) — `clr` exit code unchanged | Error Isolation |
| FT-8 | Stdout exceeding 1 MB → field truncated with `[truncated at 1MB]` marker | Truncation |
| FT-9 | `--journal-dir <cli>` + `CLR_JOURNAL_DIR=<env>` → file in CLI dir (CLI wins) | Precedence |
| FT-10 | Gate wait event emitted when `wait_for_session_slot()` blocks | Gate Emission |
| FT-11 | Validation retry event emitted on expect-strategy retry | Validation Emission |
| FT-12 | `--dry-run` does NOT create journal directory (BUG-319) | Side Effect Isolation |
| FT-13 | `--journal bogus` CLI flag → exit 1 with error | Validation |
| FT-14 | `--journal Full` (wrong case) → exit 1 | Validation |
| FT-15 | `--journal` missing value → exit 1 | Validation |
| FT-16 | `--journal full --journal meta` (last wins) → meta-level | Duplicate Handling |
| FT-17 | `--journal off --journal-dir <dir>` → no JSONL | Off Precedence |
| FT-18 | `CLR_JOURNAL=off` + `CLR_JOURNAL_DIR=<dir>` → no JSONL | Off Precedence |
| FT-19 | `CLR_JOURNAL=meta` env var controls level; stdout/stderr absent | Env Var Level |
| FT-20 | Retry event emitted with `error_class` before successful second attempt | Retry Emission |
| FT-21 | Timeout event emitted with `exit_code:4` when watchdog kills subprocess | Timeout Emission |
| FT-22 | Default journal dir resolves to `~/.clr/journal/` when no flag or env set | Default Dir |
| FT-23 | Interactive event carries `duration_ms` on the blocking (`timeout == 0`) path (AC-012, BUG-539) | Duration Emission |
| FT-24 | Interactive event carries `duration_ms` on the timeout-polling path (AC-012, BUG-539) | Duration Emission |
| FT-25 | Execution without `--dir` → `dir` == process cwd; `agent_id` composed from it (AC-018/AC-019) | Attribution |
| FT-26 | Execution with `--dir Y` → `dir` == Y preserved; `agent_id` uses Y (AC-018) | Attribution |
| FT-27 | `CLR_ACCOUNT=test.acct` → `account == "test.acct"` (env override wins, AC-020) | Attribution |
| FT-28 | Identity unresolvable → `account` absent; `user`/`host`/`agent_id` still set (AC-019/AC-020) | Attribution |
| FT-29 | Active-marker redirect seat → `account` == profile name, never a token (AC-020) | Attribution |
| FT-30 | `retry` event carries the same `account`/`agent_id` as its `execution` (AC-019/AC-020) | Attribution |
| FT-31 | Interactive from dir X → `dir` == X, `agent_id` == `{user}@{host}X/`, `account` set (AC-018–AC-020) | Attribution |

## Test Coverage Summary

- Level Default: 1 test (FT-1)
- Level Divergence: 2 tests (FT-2, FT-3)
- Default Behavior: 1 test (FT-4)
- Directory Override: 1 test (FT-5)
- Env Var: 1 test (FT-6)
- Error Isolation: 1 test (FT-7)
- Truncation: 1 test (FT-8)
- Precedence: 1 test (FT-9)
- Gate Emission: 1 test (FT-10)
- Validation Emission: 1 test (FT-11)
- Side Effect Isolation: 1 test (FT-12)
- Validation: 3 tests (FT-13, FT-14, FT-15)
- Duplicate Handling: 1 test (FT-16)
- Off Precedence: 2 tests (FT-17, FT-18)
- Env Var Level: 1 test (FT-19)
- Retry Emission: 1 test (FT-20)
- Timeout Emission: 1 test (FT-21)
- Default Dir: 1 test (FT-22)
- Duration Emission: 2 tests (FT-23, FT-24)
- Attribution: 7 tests (FT-25 through FT-31)

**Total:** 31 tests

> **Implementation note:** The actual test files use EC-N identifiers mapped to
> integration test scenarios; FT-N here is the spec-level identifier. Coverage
> spans three integration files plus one reproducer file:
> `journal_integration_test.rs` (EC-1..EC-10), `journal_integration_ext_test.rs`
> (EC-11..EC-22), `journal_attribution_test.rs` (EC-23..EC-29 = FT-25..FT-31,
> task 542), and `bug_reproducers_539_test.rs` (FT-23/FT-24, marked
> `bug_reproducer(BUG-539)`).
> CLI-wins-over-env is implemented as `ec14_journal_dir_cli_wins_over_env`;
> truncation marker as `ec15_stdout_over_1mb_has_truncation_marker`;
> gate_wait emission as `ec11_gate_wait_event_emitted_when_gate_blocks`;
> validation_retry emission as `ec12_validation_retry_event_emitted_on_expect_mismatch`.

## Architectural Constraint

FT-1 through FT-6 require a fake `claude` subprocess and a temporary directory as the journal path. All tests must set `--journal-dir <tmpdir>` (or `CLR_JOURNAL_DIR=<tmpdir>`) so that journal events land in an isolated temp directory, not `~/.clr/journal/`.

FT-7 requires a read-only directory created via `std::fs::set_permissions`. The test asserts that `clr` exits 0 despite the journal write failing.

FT-8 requires a fake `claude` subprocess that emits >1 MB of repeated output on stdout. The test reads the journal event and asserts the `stdout` field ends with `\n[truncated at 1MB]`.

---

### FT-1: `--journal full` → event with stdout field

- **Given:** temporary journal dir; fake claude that exits 0 and prints `"hello"`
- **When:** `clr -p --max-sessions 0 --journal full --journal-dir <tmpdir> "task"`
- **Then:** journal file `<tmpdir>/YYYY-MM-DD.jsonl` exists; last line parses as JSON; `event.fields.stdout` is `Some("hello")`; `event.event_type == EventType::Execution`
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-001

---

### FT-2: `--journal meta` → event without stdout/stderr

- **Given:** temporary journal dir; fake claude that exits 0 and prints `"hello"`
- **When:** `clr -p --max-sessions 0 --journal meta --journal-dir <tmpdir> "task"`
- **Then:** journal file exists; last line parses as JSON; `event.fields.stdout` is `None`; `event.fields.stderr` is `None`; `event.fields.exit_code` is `Some(0)` (metadata present)
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-002

---

### FT-3: `--journal off` → no journal event

- **Given:** temporary journal dir; fake claude that exits 0
- **When:** `clr -p --max-sessions 0 --journal off --journal-dir <tmpdir> "task"`
- **Then:** `<tmpdir>` either does not exist or contains no `.jsonl` files (journaling disabled)
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-003

---

### FT-4: No `--journal` flag → defaults to `full`

- **Given:** temporary journal dir; fake claude that exits 0 and prints `"result"`
- **When:** `clr -p --max-sessions 0 --journal-dir <tmpdir> "task"` (no --journal flag)
- **Then:** journal file exists; last line parses as JSON; `event.fields.stdout` is `Some("result")` — confirms default level is `full`
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-004

---

### FT-5: `--journal-dir <path>` → events written to specified path

- **Given:** two temporary directories: `dir_a` and `dir_b`; fake claude exits 0
- **When:** `clr -p --max-sessions 0 --journal-dir <dir_a> "task"`
- **Then:** `<dir_a>/YYYY-MM-DD.jsonl` exists and contains one event line; `<dir_b>` has no `.jsonl` files
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-005

---

### FT-6: `CLR_JOURNAL_DIR` env var routes journal events

- **Given:** temporary journal dir; env var `CLR_JOURNAL_DIR=<tmpdir>`; fake claude exits 0
- **When:** `clr -p --max-sessions 0 "task"` with `CLR_JOURNAL_DIR` set (no `--journal-dir` flag)
- **Then:** `<tmpdir>/YYYY-MM-DD.jsonl` exists and contains one event line
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-006

---

### FT-7: Journal write failure does not change exit code

- **Given:** read-only journal dir (`0o555` permissions); fake claude exits 0
- **When:** `clr -p --max-sessions 0 --journal-dir <readonly_dir> "task"`
- **Then:** exit 0 (clr exit code unchanged by journal write failure); journal event may or may not be present (best-effort)
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-008

---

### FT-8: Stdout exceeding 1 MB is truncated in journal

- **Given:** temporary journal dir; fake claude that emits >1 MB on stdout (repeated `'A'` × 1_100_000)
- **When:** `clr -p --max-sessions 0 --journal full --journal-dir <tmpdir> "task"`
- **Then:** journal event `fields.stdout` is `Some(s)` where `s.ends_with("\n[truncated at 1MB]")` and `s.len() <= 1_100_000` (truncated to 1 MB + suffix)
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-007

---

### FT-9: `--journal-dir` CLI flag wins over `CLR_JOURNAL_DIR` env var

- **Given:** two temporary directories (`cli_dir`, `env_dir`); fake claude exits 0; `CLR_JOURNAL_DIR=<env_dir>` set
- **When:** `clr -p --max-sessions 0 --journal-dir <cli_dir> "task"` with `CLR_JOURNAL_DIR=<env_dir>`
- **Then:** JSONL file appears in `<cli_dir>`; `<env_dir>` contains no `.jsonl` files (CLI flag takes precedence)
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) design — "Resolution: CLI > env > default"

---

### FT-10: Gate wait event emitted when `wait_for_session_slot()` blocks

- **Given:** ELF fake `claude` binary holding 1 gate slot for ~3 s; separate script fake for the actual subprocess; temporary journal dir
- **When:** `clr -p --max-sessions 1 --journal full --journal-dir <tmpdir> "x"` with `CLR_GATE_POLL_SECS=1`
- **Then:** JSONL contains a line with `"type":"gate_wait"` and `"gate_outcome":"acquired"`; `clr` exits 0 once gate releases
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-009

---

### FT-11: Validation retry event emitted on expect-strategy retry

- **Given:** counter-script fake `claude` (first call prints `WRONG`, second prints `RIGHT`); temporary journal dir
- **When:** `clr -p --max-sessions 0 --expect right --expect-strategy retry --retry-on-validation 1 --validation-delay 0 --journal full --journal-dir <tmpdir> "x"`
- **Then:** JSONL contains a line with `"type":"validation_retry"` (emitted before the re-attempt); second attempt matches `"right"`; `clr` exits 0
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-013

---

### FT-12: `--dry-run` does NOT create journal directory (BUG-319)

- **Given:** parent temp dir with a non-existent topic directory `must_not_exist`
- **When:** `clr --dry-run --journal-dir <parent>/must_not_exist "test"`
- **Then:** `<parent>/must_not_exist` does NOT exist on disk; dry-run output shown on stdout
- **Exit:** 0
- **Source:** BUG-319 regression guard

---

### FT-13: `--journal bogus` CLI flag exits 1

- **Given:** no special setup
- **When:** `clr --dry-run --journal bogus "test"`
- **Then:** exit 1; stderr contains `--journal` and `bogus`
- **Exit:** 1
- **Source:** [param/072_journal.md](../../../docs/cli/param/072_journal.md) — valid values: full, meta, off

---

### FT-14: `--journal Full` (case-sensitive) exits 1

- **Given:** no special setup
- **When:** `clr --dry-run --journal Full "test"` (also: FULL, Meta, META, Off, OFF)
- **Then:** exit 1 for each case variant; only lowercase accepted
- **Exit:** 1
- **Source:** [param/072_journal.md](../../../docs/cli/param/072_journal.md) — enum values are lowercase only

---

### FT-15: `--journal` missing value exits 1

- **Given:** no special setup
- **When:** `clr --dry-run --journal` (no following value)
- **Then:** exit 1; stderr mentions `--journal` or `requires a value`
- **Exit:** 1
- **Source:** parse.rs `next_value()` guard

---

### FT-16: `--journal full --journal meta` (last wins) → meta-level

- **Given:** temp journal dir; fake claude exits 0 with output
- **When:** `clr -p --max-sessions 0 --journal full --journal meta --journal-dir <tmpdir> "x"`
- **Then:** JSONL contains execution event; `stdout` field absent (meta-level wins)
- **Exit:** 0
- **Source:** Standard last-wins flag semantics

---

### FT-17: `--journal off --journal-dir <dir>` → no JSONL

- **Given:** parent temp dir with non-existent topic directory; fake claude exits 0
- **When:** `clr -p --max-sessions 0 --journal off --journal-dir <parent>/should_not_appear "x"`
- **Then:** `<parent>/should_not_appear` does NOT exist (off short-circuits before dir creation)
- **Exit:** 0
- **Source:** resolve_journal_writer() early return on "off"

---

### FT-18: `CLR_JOURNAL=off` + `CLR_JOURNAL_DIR=<dir>` → no JSONL

- **Given:** parent temp dir with non-existent topic directory; fake claude exits 0; `CLR_JOURNAL=off` + `CLR_JOURNAL_DIR=<parent>/env_off_should_not_appear`
- **When:** `clr -p --max-sessions 0 "x"` with env vars set
- **Then:** `<parent>/env_off_should_not_appear` does NOT exist
- **Exit:** 0
- **Source:** env var precedence + resolve_journal_writer() early return on "off"

---

### FT-19: `CLR_JOURNAL=meta` env var controls level; stdout/stderr absent

- **Given:** temporary journal dir; fake claude that exits 0 and prints `"env_meta_output"`; `CLR_JOURNAL=meta` env var set; no `--journal` CLI flag
- **When:** `clr -p --max-sessions 0 --journal-dir <tmpdir> "x"` with `CLR_JOURNAL=meta`
- **Then:** journal file exists; last line parses as JSON with `"type":"execution"`; `stdout` field absent; `stderr` field absent (meta level omits output fields just as `--journal meta` does)
- **Exit:** 0
- **Source:** `param/072_journal.md` — `CLR_JOURNAL` env var is the env-var counterpart to `--journal` flag; same level semantics apply

---

### FT-20: Retry event emitted with `error_class` before successful second attempt

- **Given:** counter-script fake `claude` that exits 2 on first call (classified Transient/RateLimit) and exits 0 on the second call; `--retry-on-transient 1 --transient-delay 0`; temporary journal dir
- **When:** `clr -p --retry-on-transient 1 --transient-delay 0 --max-sessions 0 --journal full --journal-dir <tmpdir> "x"`
- **Then:** exit 0 (second attempt succeeds); JSONL contains both a `"type":"retry"` event (with `"error_class":"Transient"`) and a subsequent `"type":"execution"` event; retry event appears before the execution event in the file
- **Exit:** 0
- **Source:** `feature/002_journaling_integration.md` AC-009-class — retry events emitted before each re-attempt; `--transient-delay 0` required (default 30 s causes test hang)

---

### FT-21: Timeout event emitted with `exit_code:4` when watchdog kills subprocess

- **Given:** fake `claude` that sleeps indefinitely (`sleep 300`); `_CLR_DEFAULT_TIMEOUT=2` (test-only 2 s watchdog); `--retry-override 0` (one attempt only); temporary journal dir
- **When:** `clr -p --retry-override 0 --max-sessions 0 --journal full --journal-dir <tmpdir> "x"` with `_CLR_DEFAULT_TIMEOUT=2`
- **Then:** exit 4 (watchdog killed subprocess); JSONL contains `"type":"timeout"` with `"exit_code":4`
- **Exit:** 4
- **Source:** `param/036_timeout.md` — exit 4 is the watchdog-kill exit code; `_CLR_DEFAULT_TIMEOUT` is an undocumented test-only env override (leading `_` prefix convention); `--retry-override 0` prevents the retry loop from re-attempting after watchdog fires

---

### FT-22: Default journal dir resolves to `~/.clr/journal/`

- **Given:** fresh fake HOME directory; no `CLR_JOURNAL_DIR` env var; no `--journal-dir` flag; fake claude that exits 0 and prints output
- **When:** `clr -p --max-sessions 0 "x"` with `HOME=<fake_home>`; `CLR_JOURNAL` and `CLR_JOURNAL_DIR` explicitly removed from env
- **Then:** exit 0; `<fake_home>/.clr/journal/YYYY-MM-DD.jsonl` exists and contains an execution event; no other journal location is used
- **Exit:** 0
- **Source:** `param/073_journal_dir.md` — 3-tier resolution: CLI > `CLR_JOURNAL_DIR` env > `~/.clr/journal/` default; test validates the default tier by setting `HOME` and clearing the env var

---

### FT-23: Interactive event carries `duration_ms` — blocking path (BUG-539)

- **Given:** temporary journal dir; fake claude that exits 0 immediately; no `--timeout` flag and `CLR_TIMEOUT`/`_CLR_DEFAULT_TIMEOUT` removed from env (interactive default timeout is 0 → blocking `wait()` path)
- **When:** `clr --interactive --max-sessions 0 --journal full --journal-dir <tmpdir> "x"`
- **Then:** exactly one `"type":"interactive"` line exists; it contains a `"duration_ms":<n>` key; `n < 60_000` (sanity bound — the session lasted well under a minute)
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-012; implemented as `bug_539_blocking_path_interactive_event_carries_duration_ms` in `bug_reproducers_539_test.rs`

---

### FT-24: Interactive event carries `duration_ms` — timeout-polling path (BUG-539)

- **Given:** temporary journal dir; fake claude that exits 0 immediately; `--timeout 30` (nonzero timeout → polling path with deadline)
- **When:** `clr --interactive --timeout 30 --max-sessions 0 --journal full --journal-dir <tmpdir> "x"`
- **Then:** exactly one `"type":"interactive"` line exists; it contains a `"duration_ms":<n>` key; `n < 60_000` (real elapsed time, never the timeout deadline)
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-012; implemented as `bug_539_timeout_path_interactive_event_carries_duration_ms` in `bug_reproducers_539_test.rs`

---

### FT-25: Execution without `--dir` → `dir` falls back to process cwd; `agent_id` composed from it

- **Given:** temporary journal dir; fake claude; child spawned with `.current_dir(<X>)` and no `--dir`/`--to` flag; `USER=tester`/`HOSTNAME=testhost` pinned in env
- **When:** `clr -p --max-sessions 0 --journal full --journal-dir <tmpdir> "x"` from cwd `<X>`
- **Then:** execution event has `"dir":"<X>"` (canonicalized cwd), `"user":"tester"`, `"host":"testhost"`, and `"agent_id":"tester@testhost<X>/"`
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-018/AC-019; implemented as `ec23_*` in `journal_attribution_test.rs`

---

### FT-26: Execution with explicit `--dir Y` → `dir` preserved verbatim; `agent_id` uses Y

- **Given:** temporary journal dir; fake claude; a second temp dir `<Y>` passed as `--dir <Y>` (canonicalized before passing)
- **When:** `clr -p --dir <Y> --max-sessions 0 --journal full --journal-dir <tmpdir> "x"`
- **Then:** execution event has `"dir":"<Y>"` exactly as passed (cwd fallback did NOT overwrite it) and `"agent_id":"tester@testhost<Y>/"`
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-018 (explicit values always win); implemented as `ec24_*` in `journal_attribution_test.rs`

---

### FT-27: `CLR_ACCOUNT` env override wins the account hierarchy

- **Given:** temporary journal dir; fake claude; `CLR_ACCOUNT=test.acct` set in the child env (store not consulted)
- **When:** `clr -p --max-sessions 0 --journal full --journal-dir <tmpdir> "x"`
- **Then:** execution event has `"account":"test.acct"`
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-020 (hierarchy: `CLR_ACCOUNT` first); implemented as `ec25_*` in `journal_attribution_test.rs`

---

### FT-28: Unresolvable account → `account` absent; `user`/`host`/`agent_id` still stamped

- **Given:** temporary journal dir; fake claude; `CLR_ACCOUNT` removed; `PRO` points at an empty temp root (no active-account marker anywhere)
- **When:** `clr -p --max-sessions 0 --journal full --journal-dir <tmpdir> "x"`
- **Then:** execution event contains NO `"account":` key, but `"user"`, `"host"`, and `"agent_id"` are all present
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-019/AC-020 (absent, never empty; identity fields independent of account resolution); implemented as `ec26_*` in `journal_attribution_test.rs`

---

### FT-29: Active-account marker in the credential store resolves `account`

- **Given:** temporary journal dir; fake claude; `CLR_ACCOUNT` removed; `PRO=<root>` where `<root>/.persistent/claude/credential/_active_testhost_tester` contains `kimi\n` (marker holds the profile NAME only — no token material anywhere in the fixture)
- **When:** `clr -p --max-sessions 0 --journal full --journal-dir <tmpdir> "x"`
- **Then:** execution event has `"account":"kimi"` (marker content trimmed)
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-020 (store-marker tier of the hierarchy); implemented as `ec27_*` in `journal_attribution_test.rs`

---

### FT-30: `retry` event carries the same attribution as its `execution` event

- **Given:** temporary journal dir; counter-script fake claude (exit 2 first call, exit 0 second); `--retry-on-transient 1 --transient-delay 0`; `CLR_ACCOUNT=retry.acct`
- **When:** `clr -p --retry-on-transient 1 --transient-delay 0 --max-sessions 0 --journal full --journal-dir <tmpdir> "x"`
- **Then:** both the `"type":"retry"` and `"type":"execution"` lines exist; both carry `"account":"retry.acct"` and identical `"agent_id"` values (stamping is uniform across event types)
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-019/AC-020 (every event type stamped, not just execution); implemented as `ec28_*` in `journal_attribution_test.rs`

---

### FT-31: Interactive event fully attributed (dir, agent_id, account)

- **Given:** temporary journal dir; fake claude; child spawned with `.current_dir(<X>)`; `CLR_ACCOUNT=session.acct`
- **When:** `clr --interactive --max-sessions 0 --journal full --journal-dir <tmpdir> "x"` from cwd `<X>`
- **Then:** the `"type":"interactive"` line has `"dir":"<X>"`, `"agent_id":"tester@testhost<X>/"`, and `"account":"session.acct"`
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../docs/feature/002_journaling_integration.md) AC-018–AC-020 on the interactive path; implemented as `ec29_*` in `journal_attribution_test.rs`
