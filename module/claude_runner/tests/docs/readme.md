# tests/docs

### Scope

- **Purpose**: Test-lens documentation for `claude_runner` — extends `docs/` with test planning and coverage.
- **Responsibility**: Test case indices, edge case catalogues, and coverage summaries organized to mirror `docs/`.
- **In Scope**: CLI test planning (`cli/`), feature doc tests (`feature/`), invariant doc tests (`invariant/`), API doc tests (`api/`).
- **Out of Scope**: Behavioral requirements (→ `docs/feature/`), constraints (→ `docs/invariant/`), implementation (→ `src/`).

### Conventions

- **2-digit file numbering**: Test spec files use 2-digit prefixes (`01_`, `02_`, …`27_`).
  Upstream `docs/` uses 3-digit (`001_`, `002_`). Source links from test specs reference the
  upstream 3-digit names (e.g., `../../../../docs/cli/param/001_message.md`).
- **Case ID prefixes**: Each test scope has a dedicated prefix —
  IT (command), EC (param/env_param), CC (param_group), TC (type),
  IN (invariant), FT (feature), AP (api), US (user_story).
- **Spec format**: Command specs use `- **Command:**` / `- **Expected behavior:**` / `- **Setup:**`.
  Param/type/group specs use `- **Given:**` / `- **When:**` / `- **Then:**`.
- **Commands field**: Each param and param_group spec includes `- **Commands:** run, ask` (or
  applicable subset) after the Source line, documenting which commands the parameter applies to.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `cli/` | CLI test planning: per-command, per-parameter, and per-group test case indices |
| `feature/` | Test case indices for feature doc instances in `docs/feature/` |
| `invariant/` | Test case indices for invariant doc instances in `docs/invariant/` |
| `api/` | Test case indices for API doc instances in `docs/api/` |
