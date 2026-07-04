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
| IN-6 | `cargo test -p claude_profile cli_clp_alias_test` panics on bare host before spawning any binary | Invariant holds (Rust guard L3) |
| IN-7 | `cargo test -p claude_runner` panics on bare host before `run_cli()` or any `spawn_*()` completes | Invariant holds (Rust guard L3) |
| IN-8 | `cargo test -p assistant` panics on bare host before `run_ast()` spawns any binary | Invariant holds (Rust guard L3) |
| IN-9 | `cargo test -p claude_assets` panics on bare host before `cla()` spawns any binary | Invariant holds (Rust guard L3) |
| IN-10 | `cargo test -p claude_journal_viewer` panics on bare host before any `Command::new(CLJ)` spawns | Invariant holds (Rust guard L3) |
| IN-11 | `cargo test -p claude_storage` panics on bare host before any `clg` binary spawns | Invariant holds (Rust guard L3) |
| IN-12 | `cargo test -p claude_version` panics on bare host before any `claude_version` binary spawns | Invariant holds (Rust guard L3) |
| IN-13 | `cargo test -p runbox` panics on bare host before any `crb` or `runbox` binary spawns | Invariant holds (Rust guard L3) |

**Total:** 13 IN cases

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

---

### IN-6: `cargo test -p claude_profile cli_clp_alias_test` panics on bare host

- **Given:** `cargo test -p claude_profile cli_clp_alias_test` is invoked directly on a bare host where:
  - `/.dockerenv` does not exist
  - `/run/.containerenv` does not exist
  - `RUNBOX_CONTAINER` is unset or not `"1"`
  - `VERB_LAYER` is unset or not `"l0"`
- **When:** Any alias smoke test (`a01`–`a04`) runs and calls the local `run()` helper
- **Then:** The test panics with "Tests must run inside a container" before `Command::new(bin)` spawns any process; the exit code is non-zero
- **Note:** The escape hatch `VERB_LAYER=l0 cargo test -p claude_profile cli_clp_alias_test` passes — the local `assert_container()` honors the escape hatch and the alias tests execute normally
- **Source:** [docs/invariant/009_container_only_test_execution.md](../../../docs/invariant/009_container_only_test_execution.md)

---

### IN-7: `cargo test -p claude_runner` panics on bare host before any process spawns

- **Given:** `cargo test -p claude_runner` is invoked directly on a bare host (same signals absent as IN-6)
- **When:** Any test that calls `run_cli()`, `run_cli_with_env()`, `run_dry()`, `run_ask_dry()`,
  `run_with_path()`, `spawn_fake_claude()`, `spawn_print_claude()`, `run_clr_ps()`, or
  `run_clr_kill()` in `cli_binary_test_helpers.rs` is executed
- **Then:** The test panics with "Tests must run inside a container" before `Command::new(...)` or
  `.spawn()` executes; no `clr` binary or fake claude process is spawned; exit code is non-zero
- **Note:** `VERB_LAYER=l0 cargo test -p claude_runner` passes — all 9 helpers honor the escape hatch
- **Source:** [docs/invariant/009_container_only_test_execution.md](../../../docs/invariant/009_container_only_test_execution.md)

---

### IN-8: `cargo test -p assistant` panics on bare host before any binary spawns

- **Given:** `cargo test -p assistant` is invoked directly on a bare host (same signals absent as IN-6)
- **When:** Any test that calls `run_ast()` in `tests/aggregation.rs` or `tests/cli_sanity.rs` is executed
- **Then:** The test panics with "Tests must run inside a container" before the `ast` binary is spawned; exit code is non-zero
- **Note:** `ast_package_name_is_assistant` and `ast_binary_is_present` (non-spawning path-check tests) are not guarded and run freely on host; only process-spawning tests in `run_ast()` are covered
- **Note:** `VERB_LAYER=l0 cargo test -p assistant` passes — both `run_ast()` helpers honor the escape hatch
- **Source:** [docs/invariant/009_container_only_test_execution.md](../../../docs/invariant/009_container_only_test_execution.md)

---

### IN-9: `cargo test -p claude_assets` panics on bare host before any binary spawns

- **Given:** `cargo test -p claude_assets` is invoked directly on a bare host (same signals absent as IN-6)
- **When:** Any test that calls `cla()` in `tests/cli.rs` is executed
- **Then:** The test panics with "Tests must run inside a container" before `Command::cargo_bin("cla")` spawns any process; exit code is non-zero
- **Note:** `VERB_LAYER=l0 cargo test -p claude_assets` passes — `cla()` honors the escape hatch
- **Source:** [docs/invariant/009_container_only_test_execution.md](../../../docs/invariant/009_container_only_test_execution.md)

---

### IN-10: `cargo test -p claude_journal_viewer` panics on bare host before any binary spawns

- **Given:** `cargo test -p claude_journal_viewer` is invoked directly on a bare host (same signals absent as IN-6)
- **When:** Any test that calls `run_clj()` or directly calls `Command::new(CLJ)` in `tests/viewer_integration_test.rs` is executed (includes `ec11`, `ec12`, `ec13`)
- **Then:** The test panics with "Tests must run inside a container" before the `clj` binary is spawned; exit code is non-zero
- **Note:** `claude_journal_viewer` has no `verb/` directory — the workspace-level nextest setup script (L2) covers `cargo nextest run` but not `cargo test`; L3 closes this gap
- **Note:** `VERB_LAYER=l0 cargo test -p claude_journal_viewer` passes — all 4 guard sites honor the escape hatch
- **Source:** [docs/invariant/009_container_only_test_execution.md](../../../docs/invariant/009_container_only_test_execution.md)

---

### IN-11: `cargo test -p claude_storage` panics on bare host before any binary spawns

- **Given:** `cargo test -p claude_storage` is invoked directly on a bare host (same signals absent as IN-6)
- **When:** Any test that calls `clg_cmd()` in `tests/common/mod.rs` (covers ~74 test files), or the test function `op_3_crate_compiles_after_cargo_toml_and_import_migration()` in `tests/operation_migration_guide_test.rs` is executed
- **Then:** The test panics with "Tests must run inside a container" before any `clg` binary is spawned; exit code is non-zero
- **Note:** `clg_cmd()` is the single chokepoint for all test files that import from `common/` — one guard covers ~74 files; `op_3` is guarded separately because it does not import from `common/`
- **Note:** `VERB_LAYER=l0 cargo test -p claude_storage` passes — both guard sites honor the escape hatch
- **Source:** [docs/invariant/009_container_only_test_execution.md](../../../docs/invariant/009_container_only_test_execution.md)

---

### IN-12: `cargo test -p claude_version` panics on bare host before any binary spawns

- **Given:** `cargo test -p claude_version` is invoked directly on a bare host (same signals absent as IN-6)
- **When:** Any test in the `integration` binary (calls `run_clm()` → `run_clm_with_env()` in `tests/integration/subprocess_helpers.rs`), or any test in `cli_args_test` that calls `fn run()` or one of the 6 bypass test functions (`tc494`, `tc_verbosity_level_2_verbose`, `tc_settings_key_dot_literal`, `tc_settings_key_valid_accepted`, `ec3`) that spawn directly via `Command::new(env!("CARGO_BIN_EXE_claude_version"))` is executed
- **Then:** The test panics with "Tests must run inside a container" before any `claude_version` binary is spawned; exit code is non-zero
- **Note:** `VERB_LAYER=l0 cargo test -p claude_version` passes — all guard sites in both test binaries honor the escape hatch
- **Source:** [docs/invariant/009_container_only_test_execution.md](../../../docs/invariant/009_container_only_test_execution.md)

---

### IN-13: `cargo test -p runbox` panics on bare host before any binary spawns

- **Given:** `cargo test -p runbox` is invoked directly on a bare host (same signals absent as IN-6)
- **When:** Any test that calls `crb()` or `runbox_bin()` in `tests/init_command.rs` is executed
- **Then:** The test panics with "Tests must run inside a container" before any `crb` or `runbox` binary is spawned; exit code is non-zero
- **Note:** `crb()` and `runbox_bin()` are the only two process-spawning entry points in `runbox` tests — all 14 CLI tests route through one of these helpers; no bypass sites exist
- **Note:** `VERB_LAYER=l0 cargo test -p runbox` passes — both helpers honor the escape hatch
- **Source:** [docs/invariant/009_container_only_test_execution.md](../../../docs/invariant/009_container_only_test_execution.md)
