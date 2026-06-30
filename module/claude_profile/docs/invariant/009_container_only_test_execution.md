# Invariant: Container-Only Test Execution

### Scope

- **Purpose**: Guarantee that all workspace crate tests run inside the runbox container, never directly on the host.
- **Responsibility**: Documents the container-only test execution constraint, the three detection signals, the escape hatch, and all enforcement layers across all workspace crates.
- **In Scope**: All test execution paths — `cargo nextest run`, `cargo test`, and direct layer invocations for all 19 workspace crates with tests.
- **Out of Scope**: Container build and image management (→ runbox); production binary execution (not a test concern); crates with zero test files (claud_memory).

### Invariant Statement

All tests for all workspace crates MUST execute inside the runbox container. Host-native test execution is a hard error at every enforcement layer.

**Enforcement scope by crate type:**

| Crate type | Examples | Shell layer | Nextest setup | Rust guard |
|------------|----------|-------------|---------------|------------|
| With `verb/` (17 crates) | claude_profile, assistant, assistant_kit, … | ✓ `verb/test` rejects VERB_LAYER; `l0` is hard error stub | ✓ via `filter = "all()"` | Partial — `cli_integration_test` only (via `cli_runner.rs`); other test binaries not covered |
| Without `verb/` (2 crates) | claude_journal, claude_journal_viewer | — (no verb/ dir) | ✓ via `filter = "all()"` | — |

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
| Rust guard (defense-in-depth) | Container assertion at the top of `run_cs()` in `tests/cli/cli_runner.rs` | Blocks `cargo test` **only** for `cli_integration_test`; other test binaries that do not call `run_cs()` are not covered by this layer |

**Known gap — `cargo test` partial coverage:** `cargo test` bypasses the nextest setup script (L2). The Rust guard (L3) covers only test binaries that use `cli_runner.rs` entry points. Process-spawning test binaries currently uncovered via `cargo test` (L3 absent):

- `claude_profile`: `cli_clp_alias_test` — local `run()` helper before `Command::new(bin)` → TSK-355
- `claude_runner`: `run_cli()`, `run_cli_with_env()`, `spawn_*()` in `cli_binary_test_helpers.rs` → TSK-355
- `assistant`: `run_ast()` in `aggregation.rs`; inline spawns in `cli_sanity.rs` → TSK-356
- `claude_assets`: `fn cla()` in `cli.rs` → TSK-357
- `claude_journal_viewer`: `fn run_clj()` in `viewer_integration_test.rs` → TSK-358
- `claude_storage`: `fn clg_cmd()` in `common/mod.rs` → TSK-359
- `claude_version`: `fn run_clm_with_env()` in `integration/subprocess_helpers.rs`; `fn run()` + 5 inline sites in `cli_args_test.rs` → TSK-360
- `runbox`: `fn crb()`, `fn runbox_bin()` in `init_command.rs` → TSK-361

Purely functional test binaries (`usage_integration_test`, `account_tests`, `lib_test`, etc.) are safe on host and do not require L3.

**Signal propagation:** `verb/test.d/l1` (the container-internal layer) sets `export RUNBOX_CONTAINER=1` before invoking nextest. Child processes (nextest setup script, Rust guard) inherit this variable, satisfying signal 3. Signals 1 and 2 are satisfied by the container filesystem automatically.

### Enforcement Mechanism

**`verb/test`:** Rejects any explicit `VERB_LAYER` set on the host side, because the container always invokes `test.d/l1` directly — it never goes through `verb/test`. An explicit `VERB_LAYER` on the host means someone is trying to run tests outside the container.

**`verb/test.d/l0`:** Replaced with a hard error stub. Was: host-native `w3 .test level::3`. Now: exits 1 with the standard invocation message. No host-native test execution path exists.

**`.config/setup-require-container`:** Bash script registered as a nextest setup script. Checks all three signals plus the escape hatch. Exits 1 with a clear error message if none match. Registered via `[scripts.setup.require-container]` in `.config/nextest.toml`.

**`run_cs()` in `tests/cli/cli_runner.rs`:** The primary chokepoint for integration tests. Contains an inline `assert!` that checks the three signals plus the escape hatch. Panics with the standard invocation message if executed on bare host.

### Violation Consequences

- A bare-host `cargo nextest run` exits before any test binary executes — the nextest setup script catches it
- A bare-host `cargo test -p claude_profile` panics inside `cli_integration_test` via the Rust guard; test binaries that do not use `cli_runner.rs` (e.g. `cli_clp_alias_test`, `usage_integration_test`, `account_tests`) are not covered by L3 and run freely
- A bare-host `cargo test -p claude_runner` runs completely unprotected — L3 is absent in `claude_runner`'s test helpers (see TSK-355)
- A `VERB_LAYER=l0 ./verb/test` invocation triggers `verb/test`'s VERB_LAYER rejection — the guard detects that `VERB_LAYER` is set on the host side
- Direct `./verb/test.d/l0` invocation hits the hard error stub

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `module/*/verb/test` (17 crates with `verb/`) | Outer dispatcher — rejects any `VERB_LAYER` on host |
| source | `module/*/verb/test.d/l0` (17 crates with `verb/`) | Hard error stub — host-native execution disabled |
| source | `module/*/verb/test.d/l1` (17 crates with `verb/`) | Container-internal layer — sets `RUNBOX_CONTAINER=1` |
| source | `.config/setup-require-container` | Nextest setup script — 3-signal check, workspace-wide |
| source | `.config/nextest.toml` | Nextest configuration — `filter = "all()"` |
| source | `module/claude_profile/tests/cli/cli_runner.rs` | Rust guard in `run_cs()` — defense-in-depth |
| invariant | [004_no_process_execution.md](004_no_process_execution.md) | Structural peer — zero process execution in library; all execution in container |
