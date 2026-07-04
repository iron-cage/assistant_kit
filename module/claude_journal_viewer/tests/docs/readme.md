# tests/docs

### Scope

- **Purpose**: Test-lens documentation for `claude_journal_viewer` — extends `docs/` with test planning and coverage.
- **Responsibility**: Test case indices organized to mirror `docs/feature/`, `docs/invariant/`, and `docs/cli/`.
- **In Scope**: Feature doc tests (`feature/`), invariant doc tests (`invariant/`), CLI doc tests (`cli/user_story/`, `cli/command/`, `cli/param/`, `cli/param_group/`, `cli/type/`).
- **Out of Scope**: Behavioral requirements (→ `docs/feature/`), constraints (→ `docs/invariant/`), CLI reference (→ `docs/cli/`), implementation (→ `src/`).

### Conventions

- **2-digit file numbering**: `feature/` and `invariant/` test spec files use 2-digit prefixes (`01_`, `02_`, …). Upstream `docs/feature/` and `docs/invariant/` use 3-digit (`001_`, `002_`). `cli/command/`, `cli/param/`, `cli/param_group/`, and `cli/type/` also use 2-digit prefixes, matching their upstream `docs/cli/` counterparts 1:1.
- **3-digit file numbering (cli/user_story/)**: `cli/user_story/` test spec files use 3-digit prefixes matching upstream `docs/cli/user_story/` 1:1, consistent with the cross-crate `user_story` test convention (`claude_runner`, `claude_profile`, `claude_version`).
- **Case ID prefixes**: FT (feature), IN (invariant), US (cli user story), IT (cli command), EC (cli param), CC (cli param_group), TC (cli type).
- **Spec format**: `- **Given:**` / `- **When:**` / `- **Then:**`.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `feature/` | Test case indices for feature doc instances in `docs/feature/` |
| `invariant/` | Test case indices for invariant doc instances in `docs/invariant/` |
| `cli/` | Test case indices for CLI doc instances in `docs/cli/` |
