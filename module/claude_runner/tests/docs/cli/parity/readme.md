# Parity Tests

### Scope

- **Purpose**: Document cross-command behavioral parity test cases.
- **Responsibility**: Index of per-parity-matrix test spec files covering cross-command behavioral divergence and equivalence assertions.
- **In Scope**: run/ask/isolated parity (equivalence and divergence); isolated/refresh parity (credential-operation command divergence).
- **Out of Scope**: Per-command integration tests (→ `command/`), per-parameter edge cases (→ `param/`).

Per-parity-matrix test case indices for `clr`. See [parity/](../../../../docs/cli/parity/) for specification.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| `01_run_ask_isolated.md` | Parity spec for run / ask / isolated cross-command assertions | ✅ |
| `02_isolated_refresh.md` | Parity spec for isolated / refresh cross-command assertions | ✅ |
