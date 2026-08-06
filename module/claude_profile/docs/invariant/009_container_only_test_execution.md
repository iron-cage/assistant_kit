# Invariant: Container-Only Test Execution

### Scope

- **Purpose**: Guarantee that all workspace crate tests run inside the runbox container, never directly on the host.
- **Responsibility**: Documents the container-only test execution constraint, the three detection signals, the escape hatch, and all enforcement layers across all workspace crates.
- **In Scope**: All test execution paths — `cargo nextest run`, `cargo test`, and direct layer invocations for all 18 workspace crates with tests.
- **Out of Scope**: Container build and image management (→ runbox); production binary execution (not a test concern); crates with zero test files (claud_memory).

### Invariant Statement

All tests for all workspace crates MUST execute inside the runbox container. Host-native test execution is a hard error at every enforcement layer.

**Enforcement scope by crate type:**

| Crate type | Examples | Shell layer | Nextest setup | Rust guard |
|------------|----------|-------------|---------------|------------|
| With `verb/` (16 crates) | claude_profile, assistant, assistant_kit, … | ✓ `verb/test` rejects VERB_LAYER; `l0` is hard error stub | ✓ via `filter = "all()"` | Full — `claude_profile` (2 test binaries), `claude_runner` (9 helpers), `assistant` (`run_ast()`), `claude_assets` (`cla()`), `claude_storage` (`clg_cmd()` + `op_3` direct spawn), `claude_version` (`run_clm_with_env()` + `fn run()` + 6 bypass sites) |
| Without `verb/` (2 crates) | claude_journal, claude_journal_viewer | — (no verb/ dir) | ✓ via `filter = "all()"` | `claude_journal_viewer`: `run_clj()` + ec11/ec12/ec13 guarded; `claude_journal` has no process-spawning tests |

**Measurable threshold:** Zero test runs succeed on a bare host. Any attempt exits non-zero before any test binary executes.

**Container detection — three signals (any one is sufficient):**

1. `/.dockerenv` exists (Docker-based containers)
2. `/run/.containerenv` exists (Podman-based containers — runbox uses Podman)
3. `RUNBOX_CONTAINER=1` environment variable is set (set by `verb/test.d/l1` before invoking nextest)

**Escape hatch:** `VERB_LAYER=l0` bypasses the nextest setup script and Rust guard. `verb/test` rejects ALL `VERB_LAYER` values (including `l0`) — so the escape hatch requires invoking nextest directly: `VERB_LAYER=l0 cargo nextest run`. This is a recognized developer override — not a security boundary.

**Standard invocation:** `cd module/claude_profile && ./verb/test`

### Enforcement Layers

Three independent layers ensure the invariant holds even if one layer is bypassed:

| Layer | Mechanism | Coverage |
|-------|-----------|----------|
| Shell (primary outer) | `verb/test.d/l0` exits 1 with an error message; `verb/test` rejects any `VERB_LAYER` set on the host | Blocks layer-dispatch bypass |
| Nextest setup script (primary inner) | `.config/setup-require-container` registered in `.config/nextest.toml`; runs before any test binary; exits 1 on bare host | Blocks direct `cargo nextest run` for all workspace crates |
| Rust guard (defense-in-depth) | `assert_container()` in `tests/cli/cli_runner.rs` (`run_cs()`), `tests/cli_clp_alias_test.rs`, `module/claude_runner/tests/cli_binary_test_helpers.rs` (9 helpers), `module/assistant/tests/{aggregation,cli_sanity}.rs` (`run_ast()`), `module/claude_assets/tests/cli.rs` (`cla()`), `module/claude_journal_viewer/tests/viewer_integration_test.rs` (4 sites), `module/claude_storage/tests/common/mod.rs` (`clg_cmd()`) + `operation_migration_guide_test.rs` (`op_3`), `module/claude_version/tests/integration/subprocess_helpers.rs` (`run_clm_with_env()`) + `cli_args_test.rs` (`fn run()` + 6 bypass sites) | Blocks `cargo test` for all process-spawning test binaries across the workspace |

**`cargo test` gap — fully closed:** `cargo test` bypasses the nextest setup script (L2). The Rust guard (L3) covers all process-spawning test binaries across the workspace. Process-spawning helpers guarded (TSK-355–TSK-361):
- `claude_profile`: `tests/cli/cli_runner.rs` (`run_cs()`), `tests/cli_clp_alias_test.rs` (`run()`)
- `claude_runner`: all 9 helpers in `tests/cli_binary_test_helpers.rs`
- `assistant`: `run_ast()` in `tests/aggregation.rs` and `tests/cli_sanity.rs`
- `claude_assets`: `cla()` in `tests/cli.rs`
- `claude_journal_viewer`: `run_clj()` + 3 direct spawn sites (`ec11`, `ec12`, `ec13`)
- `claude_storage`: `clg_cmd()` in `tests/common/mod.rs`; `op_3` direct spawn in `tests/operation_migration_guide_test.rs`
- `claude_version`: `run_clm_with_env()` in `tests/integration/subprocess_helpers.rs`; `fn run()` + 6 bypass sites in `tests/cli_args_test.rs`

Purely functional test binaries (`usage_integration_test`, `account_tests`, `lib_test`, etc.) are safe on host and do not require L3.

**Signal propagation:** `verb/test.d/l1` (the container-internal layer) sets `export RUNBOX_CONTAINER=1` before invoking nextest. Child processes (nextest setup script, Rust guard) inherit this variable, satisfying signal 3. Signals 1 and 2 are satisfied by the container filesystem automatically.

### Enforcement Mechanism

**`verb/test`:** Rejects any explicit `VERB_LAYER` set on the host side, because the container always invokes `test.d/l1` directly — it never goes through `verb/test`. An explicit `VERB_LAYER` on the host means someone is trying to run tests outside the container.

**`verb/test.d/l0`:** Replaced with a hard error stub. Was: host-native `w3 .test level::3`. Now: exits 1 with the standard invocation message. No host-native test execution path exists.

**`.config/setup-require-container`:** Bash script registered as a nextest setup script. Checks all three signals plus the escape hatch. Exits 1 with a clear error message if none match. Registered via `[scripts.setup.require-container]` in `.config/nextest.toml`.

**`run_cs()` in `tests/cli/cli_runner.rs`:** The primary chokepoint for integration tests. Contains an inline `assert!` that checks the three signals plus the escape hatch. Panics with the standard invocation message if executed on bare host.

### Violation Consequences

- A bare-host `cargo nextest run` exits before any test binary executes — the nextest setup script catches it
- A bare-host `cargo test -p claude_profile` panics inside both `cli_integration_test` (via `cli_runner.rs`) and `cli_clp_alias_test` (via local `assert_container()`); purely functional test binaries (`usage_integration_test`, `account_tests`) are not covered by L3 and run freely
- A bare-host `cargo test -p claude_runner` panics inside any test that calls a process-spawning helper in `cli_binary_test_helpers.rs` — L3 is now present in all 9 helpers
- A bare-host `cargo test -p assistant` panics inside any test that calls `run_ast()` in `aggregation.rs` or `cli_sanity.rs` — L3 is now present in both helpers
- A bare-host `cargo test -p claude_assets` panics inside any test that calls `cla()` in `cli.rs` — L3 is now present in the helper
- A bare-host `cargo test -p claude_journal_viewer` panics before `run_clj()` or any direct `Command::new(CLJ)` (ec11/ec12/ec13) completes — L3 is now present in all 4 spawn sites
- A bare-host `cargo test -p claude_storage` panics inside any test that calls `clg_cmd()` (via `common/mod.rs`) or the direct spawn in `op_3_crate_compiles_after_cargo_toml_and_import_migration()` — L3 is now present in both sites
- A bare-host `cargo test -p claude_version` panics inside any test that calls `run_clm()` (via `subprocess_helpers.rs`), `run()` in `cli_args_test.rs`, or any of the 6 bypass test functions — L3 is now present in all sites
- A `VERB_LAYER=l0 ./verb/test` invocation triggers `verb/test`'s VERB_LAYER rejection — the guard detects that `VERB_LAYER` is set on the host side
- Direct `./verb/test.d/l0` invocation hits the hard error stub

### Sources

| File | Relationship |
|------|-------------|
| `module/*/verb/test` (16 crates with `verb/`) | Outer dispatcher — rejects any `VERB_LAYER` on host |
| `module/*/verb/test.d/l0` (16 crates with `verb/`) | Hard error stub — host-native execution disabled |
| `module/*/verb/test.d/l1` (16 crates with `verb/`) | Container-internal layer — sets `RUNBOX_CONTAINER=1` |
| `.config/setup-require-container` | Nextest setup script — 3-signal check, workspace-wide |
| `.config/nextest.toml` | Nextest configuration — `filter = "all()"` |

### Tests

| File | Relationship |
|------|-------------|
| `module/claude_profile/tests/cli/cli_runner.rs` | Rust guard in `run_cs()` — defense-in-depth |
| `module/claude_profile/tests/cli_clp_alias_test.rs` | Rust guard in local `run()` — defense-in-depth |
| `module/claude_runner/tests/cli_binary_test_helpers.rs` | Rust guard in 9 process-spawning helpers — defense-in-depth |
| `module/assistant/tests/aggregation.rs` | Rust guard in `run_ast()` — defense-in-depth |
| `module/assistant/tests/cli_sanity.rs` | Rust guard in `run_ast()` helper — defense-in-depth |
| `module/claude_assets/tests/cli.rs` | Rust guard in `cla()` — defense-in-depth |
| `module/claude_journal_viewer/tests/viewer_integration_test.rs` | Rust guard in `run_clj()` + ec11/ec12/ec13 — defense-in-depth |
| `module/claude_storage/tests/common/mod.rs` | Rust guard in `clg_cmd()` — covers all ~74 claude_storage test files |
| `module/claude_storage/tests/operation_migration_guide_test.rs` | Rust guard in `op_3` direct spawn — defense-in-depth |
| `module/claude_version/tests/integration/subprocess_helpers.rs` | Rust guard in `run_clm_with_env()` — covers all integration tests |
| `module/claude_version/tests/cli_args_test.rs` | Rust guard in `fn run()` + 6 bypass test functions — defense-in-depth |

### Invariants

| File | Relationship |
|------|-------------|
| [004_no_process_execution.md](004_no_process_execution.md) | Structural peer — zero process execution in library; all execution in container |
