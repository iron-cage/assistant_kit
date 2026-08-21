# Test: `topic`

Integration test planning for the `topic` command. See [command/11_topic.md](../../../../docs/cli/command/11_topic.md) for specification.

`topic` is a `run`/`ask` alias that overrides `--topic`'s default with a generated slug.
Tests focus on slug generation, disambiguation, and delegation to `run`'s execution path.
New topics default to fork mode (deterministic UUIDv5 session file, no `-slug` directory);
explicit `--from` forces dir mode — IT-4/IT-5 exercise the dir-mode transplant path that
way. Fork-mode first-use/repeat-use mechanics (F01–F18) are planned separately in
[param/088_topic_mode.md](../param/088_topic_mode.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clr topic "msg"` dry-run output contains a generated topic slug (fork-mode preview line) | Slug Generation |
| IT-2 | Two auto-named `clr topic` calls with the same message produce distinct topics | Disambiguation |
| IT-3 | `clr topic --topic NAME "msg"` dry-run identical to `clr ask --topic NAME "msg"` dry-run | Equivalence |
| IT-4 | `clr topic --topic NAME --from SRC "msg"` (first call; `--from` forces dir mode) plans a session-transplant clone | Clone |
| IT-5 | `clr topic --topic NAME --from SRC "msg"` (second call, same NAME) re-clones the stale copy by default, then continues via `-c` (`--keep-clone` preserves it) | Continue |
| IT-6 | Unknown flag → exit 1, error message | Error Handling |
| IT-7 | `clr topic help` → dispatches to help, exit 0 | Help |
| IT-8 | `clr topic --dry-run --effort high "msg"` → contains `--effort high` | Param Passthrough |

## Test Coverage Summary

- Slug Generation: 1 test (IT-1)
- Disambiguation: 1 test (IT-2)
- Equivalence: 1 test (IT-3)
- Clone: 1 test (IT-4)
- Continue: 1 test (IT-5)
- Error Handling: 1 test (IT-6)
- Help: 1 test (IT-7)
- Param Passthrough: 1 test (IT-8)

**Total:** 8 tests

---

### IT-1: Auto-generated topic slug appears in dry-run output

- **Command:** `clr topic --dry-run "Investigate the flaky concurrency-gate test"`
- **Expected behavior:** stdout contains a `# topic-fork: topic=<slug> session=<uuid> source=fresh base=<dir>` preview line where `<slug>` is derived from the message text (non-empty, lowercase, hyphenated) — new topics default to fork mode, so no `/-<slug>` directory path appears
- **Exit:** 0
- **Source:** [command/11_topic.md](../../../../docs/cli/command/11_topic.md)

---

### IT-2: Repeated auto-naming disambiguates via counter

- **Setup:** the first slug's topic directory exists on disk (pre-created fixture dir simulating a prior dir-mode claim)
- **Command:** `clr topic --dry-run "Investigate the flaky concurrency-gate test"` (same message, second call)
- **Expected behavior:** stdout's `# topic-fork:` preview line names `<slug>-2` (or next free counter) — the pre-existing `-<slug>` directory marks the first name taken (one of three freshness signals: dir exists, dir-mode storage has a qualifying session, fork session file non-empty); the fresh disambiguated slug itself starts in fork mode
- **Exit:** 0
- **Source:** [command/11_topic.md](../../../../docs/cli/command/11_topic.md)

---

### IT-3: Explicit `--topic` makes `topic` identical to `ask`

- **Command:** `clr topic --dry-run --topic auth-refactor "q"` vs `clr ask --dry-run --topic auth-refactor "q"`
- **Expected behavior:** Both produce identical stdout (same assembled command, same effective dir)
- **Exit:** 0
- **Source:** [command/11_topic.md](../../../../docs/cli/command/11_topic.md), [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-4: First call to an explicit topic name plans a clone (dir mode via `--from`)

- **Given:** source dir named by `--from` has a qualifying `.jsonl` session file; target topic directory does not yet exist
- **Command:** `clr topic --topic fresh-topic --from <src> "Start this"` — explicit `--from` forces dir mode (a fork-mode session file has nothing to transplant into a directory)
- **Expected behavior:** the session-transplant plan fires: the source session `.jsonl` is copied byte-identically into the new topic directory's storage
- **Exit:** 0
- **Source:** [command/11_topic.md](../../../../docs/cli/command/11_topic.md), [param/076_from.md](../../../../docs/cli/param/076_from.md), [param/088_topic_mode.md](../../../../docs/cli/param/088_topic_mode.md)

---

### IT-5: Second call to the same explicit topic name continues (dir mode via `--from`)

- **Given:** target topic directory from IT-4 now has its own (possibly diverged) session file
- **Command:** `clr topic --topic fresh-topic --from <src> "Continue this"` — same dir-mode forcing as IT-4
- **Expected behavior:** the explicit `--from` re-clones by default — the stale destination copy is overwritten with a fresh copy of the source, announced on stderr (`re-cloning over existing session copy`); the conversation then continues via `-c`. `--keep-clone` (or `CLR_KEEP_CLONE=1`) preserves the diverged destination instead — see [param/089_keep_clone.md](../param/089_keep_clone.md) KC-2/KC-3
- **Exit:** 0
- **Source:** [command/11_topic.md](../../../../docs/cli/command/11_topic.md), [param/076_from.md](../../../../docs/cli/param/076_from.md) step 7, [param/089_keep_clone.md](../../../../docs/cli/param/089_keep_clone.md)

---

### IT-6: Unknown flag → exit 1

- **Command:** `clr topic --unknown-flag "message"`
- **Expected behavior:** Stderr contains "unknown option"; exit code 1
- **Exit:** 1
- **Source:** [command/11_topic.md](../../../../docs/cli/command/11_topic.md)

---

### IT-7: `clr topic help` → dispatches to help

- **Command:** `clr topic help`
- **Expected behavior:** stdout contains usage information; exit code 0
- **Exit:** 0
- **Source:** [command/11_topic.md](../../../../docs/cli/command/11_topic.md)

---

### IT-8: `--effort high` passed through correctly

- **Command:** `clr topic --dry-run --effort high "message"`
- **Expected behavior:** Command line contains `--effort high`
- **Exit:** 0
- **Source:** [command/11_topic.md](../../../../docs/cli/command/11_topic.md)
