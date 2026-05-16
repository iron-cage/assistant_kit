# tests/docs

### Scope

- **Purpose**: Test-lens documentation for `claude_runner` — extends `docs/` with test planning and coverage.
- **Responsibility**: Test case indices, edge case catalogues, and coverage summaries organized to mirror `docs/`.
- **In Scope**: CLI test planning (`cli/`), feature doc tests (`feature/`), invariant doc tests (`invariant/`), API doc tests (`api/`).
- **Out of Scope**: Behavioral requirements (→ `docs/feature/`), constraints (→ `docs/invariant/`), implementation (→ `src/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `cli/` | CLI test planning: per-command, per-parameter, and per-group test case indices |
| `feature/` | Test case indices for feature doc instances in `docs/feature/` |
| `invariant/` | Test case indices for invariant doc instances in `docs/invariant/` |
| `api/` | Test case indices for API doc instances in `docs/api/` |
