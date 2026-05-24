# Feature Test Specs

### Scope

- **Purpose**: Feature-level test specifications mirroring `docs/feature/`.
- **Responsibility**: FT-prefixed test case specs for each feature doc instance in `docs/feature/`.
- **In Scope**: One spec file per `docs/feature/` instance, covering functional behavior and acceptance criteria.
- **Out of Scope**: Per-command integration tests (→ `../cli/command/`), per-parameter edge cases (→ `../cli/param/`).

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| `01_cli_tool.md` | FT- test cases for the CLI tool feature (`docs/feature/001_cli_tool.md`) | ✅ |
