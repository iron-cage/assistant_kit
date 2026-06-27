# tests/docs

### Scope

- **Purpose**: Test-lens documentation for `claude_journal_viewer` — extends `docs/` with test planning and coverage.
- **Responsibility**: Test case indices organized to mirror `docs/feature/` and `docs/invariant/`.
- **In Scope**: Feature doc tests (`feature/`), invariant doc tests (`invariant/`).
- **Out of Scope**: Behavioral requirements (→ `docs/feature/`), constraints (→ `docs/invariant/`), implementation (→ `src/`).

### Conventions

- **2-digit file numbering**: Test spec files use 2-digit prefixes (`01_`, `02_`, …). Upstream `docs/` uses 3-digit (`001_`, `002_`).
- **Case ID prefixes**: FT (feature), IN (invariant).
- **Spec format**: `- **Given:**` / `- **When:**` / `- **Then:**`.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `feature/` | Test case indices for feature doc instances in `docs/feature/` |
| `invariant/` | Test case indices for invariant doc instances in `docs/invariant/` |
