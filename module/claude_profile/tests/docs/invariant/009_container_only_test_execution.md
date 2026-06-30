# Test: Invariant 009 — Container-Only Test Execution

Property assertion cases for `docs/invariant/009_container_only_test_execution.md`. Verifies that
all test execution paths enforce the container requirement and that the escape hatch works correctly.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | Nextest setup script exits 1 on bare host (no container signals, no escape hatch) | Invariant holds (normal) |
| IN-2 | `VERB_LAYER=l0` escape hatch causes setup script to pass | Invariant holds (boundary) |
| IN-3 | `RUNBOX_CONTAINER=1` signal satisfies setup script | Invariant holds (signal 3) |
| IN-4 | `verb/test` rejects any `VERB_LAYER` value — including `l0` | Invariant holds (shell outer) |
| IN-5 | `verb/test.d/l0` exits 1 as hard-error stub | Invariant holds (shell l0) |

**Total:** 5 IN cases

---

### IN-1: Nextest setup script exits 1 on bare host

- **Given:** `.config/setup-require-container` is executed in an environment where:
  - `/.dockerenv` does not exist
  - `/run/.containerenv` does not exist
  - `RUNBOX_CONTAINER` is unset or not `"1"`
  - `VERB_LAYER` is unset or not `"l0"`
- **When:** The script is run directly via `bash .config/setup-require-container`
- **Then:** The script exits with code 1 and writes an error message to stderr containing
  "Tests must run inside a container" and the standard invocation hint
  (`cd module/claude_profile && ./verb/test`)
- **Source:** [docs/invariant/009_container_only_test_execution.md](../../../docs/invariant/009_container_only_test_execution.md)

---

### IN-2: `VERB_LAYER=l0` escape hatch passes the setup script

- **Given:** `.config/setup-require-container` is executed in an environment where no container
  signals are present (`/.dockerenv` absent, `/run/.containerenv` absent, `RUNBOX_CONTAINER` unset)
  but `VERB_LAYER=l0` is set
- **When:** The script is run directly via `VERB_LAYER=l0 bash .config/setup-require-container`
- **Then:** The script exits with code 0 — the escape hatch is recognized and the container check
  is bypassed without error
- **Source:** [docs/invariant/009_container_only_test_execution.md](../../../docs/invariant/009_container_only_test_execution.md)

---

### IN-3: `RUNBOX_CONTAINER=1` signal satisfies setup script

- **Given:** `.config/setup-require-container` is executed with `RUNBOX_CONTAINER=1` set in the
  environment but no container filesystem signals (`/.dockerenv` absent, `/run/.containerenv` absent)
  and `VERB_LAYER` unset
- **When:** The script is run directly via `RUNBOX_CONTAINER=1 bash .config/setup-require-container`
- **Then:** The script exits with code 0 — signal 3 is sufficient; the script does not require
  filesystem signals to confirm container presence
- **Source:** [docs/invariant/009_container_only_test_execution.md](../../../docs/invariant/009_container_only_test_execution.md)

---

### IN-4: `verb/test` rejects any `VERB_LAYER` value — including `l0`

- **Given:** `verb/test` is invoked on a bare host with `VERB_LAYER=l0` set in the environment
- **When:** `VERB_LAYER=l0 bash ./verb/test 2>&1; echo "exit:$?"`
- **Then:** The script exits with code 1 and writes to stderr a message containing
  `"VERB_LAYER is not valid on the host side"` — `verb/test` treats all `VERB_LAYER` values as
  bypass attempts; `l0` is not a special-case exception at the `verb/test` level
- **Note:** The authorized escape hatch for host development is `VERB_LAYER=l0 cargo nextest run`
  (bypasses `verb/test` entirely and invokes nextest directly; the setup script honors `VERB_LAYER=l0`)
- **Source:** [docs/invariant/009_container_only_test_execution.md](../../../docs/invariant/009_container_only_test_execution.md)

---

### IN-5: `verb/test.d/l0` exits 1 as a hard-error stub

- **Given:** `verb/test.d/l0` is invoked directly on a bare host (no container signals, no escape hatch)
- **When:** `bash ./verb/test.d/l0 2>&1; echo "exit:$?"`
- **Then:** The script exits with code 1 and writes to stderr a message containing
  `"host-native test execution (l0) is disabled"` — the `l0` layer is a tombstoned stub with no
  active host-native execution path; it does not invoke `w3` or any test runner
- **Source:** [docs/invariant/009_container_only_test_execution.md](../../../docs/invariant/009_container_only_test_execution.md)
