# Test: `topic`

Integration test planning for the `topic` command. See [command/11_topic.md](../../../../docs/cli/command/11_topic.md) for specification.

`topic` is a `run`/`ask` alias that overrides `--subdir`'s default with a generated slug.
Tests focus on slug generation, disambiguation, and delegation to `run`'s execution path.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clr topic "msg"` dry-run output contains a generated `--subdir` path | Slug Generation |
| IT-2 | Two auto-named `clr topic` calls with the same message produce distinct subdirs | Disambiguation |
| IT-3 | `clr topic --subdir NAME "msg"` dry-run identical to `clr ask --subdir NAME "msg"` dry-run | Equivalence |
| IT-4 | `clr topic --subdir NAME "msg"` (first call) plans a session-transplant clone | Clone |
| IT-5 | `clr topic --subdir NAME "msg"` (second call, same NAME) continues via `-c`, no re-clone | Continue |
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

### IT-1: Auto-generated `--subdir` path appears in dry-run output

- **Command:** `clr topic --dry-run "Investigate the flaky concurrency-gate test"`
- **Expected behavior:** stdout contains a path ending in `/-<slug>` where `<slug>` is derived from the message text (non-empty, lowercase, hyphenated)
- **Exit:** 0
- **Source:** [command/11_topic.md](../../../../docs/cli/command/11_topic.md)

---

### IT-2: Repeated auto-naming disambiguates via counter

- **Setup:** run IT-1's command once for real (non-dry-run, or with a shared fixture dir) so the first slug's subdirectory exists on disk
- **Command:** `clr topic --dry-run "Investigate the flaky concurrency-gate test"` (same message, second call)
- **Expected behavior:** stdout contains a path ending in `/-<slug>-2` (or next free counter) — distinct from the first call's path
- **Exit:** 0
- **Source:** [command/11_topic.md](../../../../docs/cli/command/11_topic.md)

---

### IT-3: Explicit `--subdir` makes `topic` identical to `ask`

- **Command:** `clr topic --dry-run --subdir auth-refactor "q"` vs `clr ask --dry-run --subdir auth-refactor "q"`
- **Expected behavior:** Both produce identical stdout (same assembled command, same effective dir)
- **Exit:** 0
- **Source:** [command/11_topic.md](../../../../docs/cli/command/11_topic.md), [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-4: First call to an explicit topic name plans a clone

- **Given:** source dir (cwd) has a qualifying `.jsonl` session file; target subdirectory does not yet exist
- **Command:** `clr topic --dry-run --subdir fresh-topic "Start this"`
- **Expected behavior:** dry-run output includes a `# session-transplant:` plan line copying the source session into the new subdirectory's storage
- **Exit:** 0
- **Source:** [command/11_topic.md](../../../../docs/cli/command/11_topic.md), [param/076_from.md](../../../../docs/cli/param/076_from.md)

---

### IT-5: Second call to the same explicit topic name continues

- **Given:** target subdirectory from IT-4 now has its own session file (from the first, non-dry-run call)
- **Command:** `clr topic --dry-run --subdir fresh-topic "Continue this"`
- **Expected behavior:** dry-run output does NOT include a `# session-transplant:` plan line (source and target storage already match); output DOES include `-c "` — the topic's own conversation continues
- **Exit:** 0
- **Source:** [command/11_topic.md](../../../../docs/cli/command/11_topic.md)

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
