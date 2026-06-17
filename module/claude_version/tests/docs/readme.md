# tests/docs

### Scope

- **Purpose**: Test-lens documentation for `claude_version` — extends `docs/` with test planning and coverage.
- **Responsibility**: Test case indices, edge case catalogues, and coverage summaries organized to mirror `docs/`.
- **In Scope**: CLI test planning (`cli/`), algorithm test surfaces (`algorithm/`), feature test surfaces (`feature/`), pattern test surfaces (`pattern/`), catalog test surfaces (`catalog/`), pitfall test surfaces (`pitfall/`).
- **Out of Scope**: Behavioral requirements (→ `docs/feature/`), implementation (→ `src/`).

| Directory | Responsibility |
|-----------|----------------|
| `cli/` | CLI test planning: per-command, per-parameter, per-type, per-group, and per-user-story test case indices |
| `feature/` | Feature test surface: per-feature FT- test case specifications |
| `pattern/` | Pattern test surface: per-pattern PT- test case specifications |
| `algorithm/` | Algorithm test surface: per-algorithm AC- test case specifications |
| `catalog/` | Collection test surface: per-decision DD- test case specifications |
| `pitfall/` | Pitfall test surface: per-pitfall PF- test case specifications |
