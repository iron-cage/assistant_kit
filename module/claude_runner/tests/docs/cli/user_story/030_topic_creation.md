# User Story :: Topic Creation

Test case spec for [030_topic_creation.md](../../../../docs/cli/user_story/030_topic_creation.md).

## Test Case Index

| ID | Test Name | AC | Status |
|----|-----------|-----|-----|
| US-1 | Auto-named topic generates a slug from the message | AC-001 | ✅ |
| US-2 | Auto-naming disambiguates repeated calls via counter | AC-001 | ✅ |
| US-3 | Explicit `--topic` bypasses slug generation | AC-002 | ✅ |
| US-4 | First use of a topic name clones the source session | AC-003 | ✅ |
| US-5 | Repeat use of the same topic name continues its conversation | AC-004 | ✅ |
| US-6 | `topic` accepts every `run`/`ask` parameter unchanged | AC-006 | ✅ partial |

AC-005 (no new session-management code) has no test case in this index — it is an
architectural claim about `topic`'s implementation, not an observable CLI behavior.
US-6 is marked partial: see its own **Not covered** note.

---

### US-1: Auto-named topic generates a slug from the message

- **Given:** neither a topic directory nor session storage matching the message's slug exists under the effective base
- **When:** `clr topic --dry-run "Investigate the flaky concurrency-gate test"`
- **Then:** dry-run output contains a `# topic-fork: topic=investigate…` preview line carrying the auto-generated slug — a brand-new topic plans in fork mode, so the slug surfaces in the preview line rather than as a `-slug` directory path
- **Exit:** 0
- **Verifies:** AC-001
- **Implemented by:** `topic_command_test.rs::t01_auto_generated_topic_shown_in_dry_run`

---

### US-2: Auto-naming disambiguates repeated calls via counter

- **Given:** a `-flaky-gate-test` topic directory already exists under the `--dir` base (freshness signal 1: directory existence, from a prior real invocation)
- **When:** `clr topic --dry-run --dir <base> "flaky gate test"`
- **Then:** dry-run output contains `# topic-fork: topic=flaky-gate-test-2 ` — the `-2` counter suffix disambiguates past the taken name, and the fresh disambiguated slug is itself a new topic and therefore fork-mode
- **Exit:** 0
- **Verifies:** AC-001
- **Implemented by:** `topic_command_test.rs::t02_repeated_auto_naming_disambiguates_via_counter`

---

### US-3: Explicit `--topic` bypasses slug generation

- **Given:** clean cwd
- **When:** `clr topic --dry-run --topic auth-refactor "q"`, compared against `clr ask --dry-run --topic auth-refactor "q"`
- **Then:** the two dry-run outputs are byte-identical — no slug is derived from the message text, and no counter suffix is appended
- **Exit:** 0
- **Verifies:** AC-002
- **Implemented by:** `topic_command_test.rs::t03_explicit_topic_matches_ask_byte_for_byte`

---

### US-4: First use of a topic name clones the source session

- **Given:** `--from`'s source project has a qualifying `.jsonl` session in `CLAUDE_HOME` storage; the `-521-topic-clone` topic directory has none of its own
- **When:** `clr topic --dir <base> --topic 521-topic-clone --from <src> "clone this session please"` — a real (non-dry-run) invocation against a stubbed `claude` executable
- **Then:** the source `.jsonl` is copied byte-identically into the topic directory's own storage before spawn, and the source session itself is left unmodified
- **Exit:** 0
- **Verifies:** AC-003
- **Implemented by:** `topic_command_test.rs::transplant::t04_first_explicit_topic_call_clones_session` (`#[cfg(unix)]`)

---

### US-5: Repeat use of the same topic name continues its conversation

- **Given:** a first `clr topic --topic 541-drift` call already cloned cwd's then-current session A into the topic's own storage; cwd has since gained a newer, unrelated session B of its own
- **When:** a second `clr topic --topic 541-drift "continue the topic"` from the same cwd — `--from` omitted, real (non-dry-run) invocation against a stubbed `claude` executable
- **Then:** session B is NOT transplanted into the topic's storage; session A remains present and is the one continued — the topic's continuity does not depend on cwd's most-recent-session drift
- **Exit:** 0
- **Verifies:** AC-004
- **Implemented by:** `topic_command_test.rs::transplant::t09_second_auto_topic_call_ignores_unrelated_source_session_drift` (`#[cfg(unix)]`, `bug_reproducer(BUG-541)`)

---

### US-6: `topic` accepts every `run`/`ask` parameter unchanged

- **Given:** clean cwd
- **When:** `clr topic --dry-run --topic effort-check --effort high "msg"`, compared against the same `clr ask` invocation
- **Then:** the two dry-run outputs are byte-identical and visibly contain `high` — a parameter that would change dry-run output if passthrough broke, so equivalence is not vacuous
- **Exit:** 0
- **Verifies:** AC-006
- **Implemented by:** `topic_command_test.rs::t08_effort_high_passthrough_matches_ask_dry_run`
- **Not covered:** the test pins `--effort` as the single representative parameter. AC-006's full claim — *every* `run`/`ask` parameter accepted with an identical default — has no enumerating test; a second parameter (e.g. `--model`) is not exercised through `topic` at all.
