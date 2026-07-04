# Testing

### Scope

- **Purpose**: Document test case plans for the `clj` CLI doc entity (`docs/cli/`).
- **Responsibility**: Index of per-doc-type test case planning directories mirroring `docs/cli/`.
- **In Scope**: User story, command, param, param_group, and type test specs mirroring all 5 `docs/cli/` doc-type directories.
- **Out of Scope**: Automated test implementations (→ `tests/` in crate), spec documentation (→ `docs/cli/`).

Test case planning for `clj` CLI. Each file contains a Test Case Index with coverage summary.

### Responsibility Table

| Directory | Responsibility |
|-----------|----------------|
| `user_story/` | Per-user-story end-to-end workflow test specs |
| `command/` | Per-command IT- test specs (8 files) |
| `param/` | Per-parameter EC- edge case test specs (28 files) |
| `param_group/` | Per-group CC- interaction test specs (5 files) |
| `type/` | Per-type TC- validation test specs (11 files) |
