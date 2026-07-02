# Invariant: Container-Only Test Execution

### Scope

- **Purpose**: Guarantee that all `claude_runner` tests run inside the runbox container, never on the host.
- **Responsibility**: Documents the container-only test execution constraint, the two enforcement layers, the detection signals, and the escape hatch for this crate.
- **In Scope**: All test execution paths — `cargo nextest run` and direct `verb/test` invocation for the `claude_runner` crate.
- **Out of Scope**: Container build and image management (→ runbox); other workspace crates (→ `claude_profile/docs/invariant/009_container_only_test_execution.md` covers all 19 crates); production binary execution.

### Invariant Statement

All tests for `claude_runner` MUST execute inside the runbox container. Host-native test execution is a hard error at every enforcement layer.

**Measurable threshold:** Zero test runs succeed on a bare host. Any attempt exits non-zero before any test binary executes.

**Container detection — three signals (any one is sufficient):**

1. `/.dockerenv` exists (Docker-based containers)
2. `/run/.containerenv` exists (Podman-based containers — runbox uses Podman)
3. `RUNBOX_CONTAINER=1` environment variable is set (set by `verb/test.d/l1` before invoking nextest)

**Escape hatch:** Direct `VERB_LAYER=l0 cargo nextest run` bypasses both layers. `verb/test` rejects all `VERB_LAYER` values — so the escape hatch requires invoking nextest directly. Recognized developer override; not a security boundary.

**Standard invocation:** `cd module/claude_runner && ./verb/test`

### Enforcement Layers

Two independent layers ensure the invariant holds:

| Layer | Mechanism | Coverage |
|-------|-----------|----------|
| Shell (outer) | `verb/test.d/l0` exits 1 with an error message; `verb/test` rejects any `VERB_LAYER` set on the host | Blocks layer-dispatch bypass |
| Nextest setup script (inner) | `.config/setup-require-container` registered in `.config/nextest.toml`; runs before any test binary; exits 1 on bare host | Blocks direct `cargo nextest run` |

Note: `claude_runner` has no Rust guard layer (unlike `claude_profile` which adds a defense-in-depth assertion in `run_cs()`). The two-layer enforcement is sufficient for this crate because `claude_runner` tests use `cli_binary_test_helpers.rs` rather than a dedicated runner struct.

**Signal propagation:** `verb/test.d/l1` sets `export RUNBOX_CONTAINER=1` before invoking nextest. The nextest setup script inherits this variable, satisfying signal 3. Signals 1 and 2 are satisfied by the container filesystem automatically.

### Enforcement Mechanism

**`verb/test`:** Rejects any explicit `VERB_LAYER` set on the host side. An explicit `VERB_LAYER` on the host means someone is trying to run tests outside the container.

**`verb/test.d/l0`:** Hard error stub. Exits 1 with the standard invocation message. No host-native test execution path exists.

**`.config/setup-require-container`:** Workspace-wide bash script registered as a nextest setup script. Checks all three signals plus the escape hatch. Exits 1 with a clear error message if none match.

### Violation Consequences

- A bare-host `cargo nextest run` exits before any test binary executes — the nextest setup script catches it
- A `VERB_LAYER=l0 ./verb/test` invocation triggers `verb/test`'s `VERB_LAYER` rejection
- Direct `./verb/test.d/l0` invocation hits the hard error stub

### Tests

| File | Notes |
|------|-------|
| `../../tests/invariant_container_test.rs` | IT-1–IT-5: nextest.toml setup script registration (IT-1); setup-require-container existence (IT-2); three-signal coverage (IT-3–IT-5) |

### Sources

| File | Relationship |
|------|--------------|
| `verb/test` | Outer dispatcher — rejects any `VERB_LAYER` on host |
| `verb/test.d/l0` | Hard error stub — host-native execution disabled |
| `verb/test.d/l1` | Container-internal layer — sets `RUNBOX_CONTAINER=1` |
| `.config/setup-require-container` | Nextest setup script — 3-signal check, workspace-wide |
| `.config/nextest.toml` | Nextest configuration — `filter = "all()"` |

### Invariants

| File | Relationship |
|------|--------------|
| `claude_profile/docs/invariant/009_container_only_test_execution.md` | Workspace-wide invariant — full enforcement story for all 19 crates |

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](../feature/001_runner_tool.md) | Runner tool design — this invariant governs `claude_runner` test execution |
