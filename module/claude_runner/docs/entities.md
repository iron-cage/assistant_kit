# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `feature/` | Runner tool design: execution modes, defaults, YAML library | [feature/readme.md](feature/readme.md) | 1 |
| `invariant/` | Default flag injection and dependency constraint rules | [invariant/readme.md](invariant/readme.md) | 2 |
| `api/` | Public library API contracts | [api/readme.md](api/readme.md) | 1 |
| `cli/testing/command/` | Per-command integration test case indices (semantic naming) | [cli/testing/command/readme.md](cli/testing/command/readme.md) | 2 |
| `cli/testing/param/` | Per-parameter edge case indices (semantic naming) | [cli/testing/param/readme.md](cli/testing/param/readme.md) | 2 |
| `cli/testing/param_group/` | Per-parameter-group integration test indices (semantic naming) | [cli/testing/param_group/readme.md](cli/testing/param_group/readme.md) | 1 |

**Note:** `cli/testing/command/`, `cli/testing/param/`, and `cli/testing/param_group/` use semantic filenames. Their instances are excluded from the Master Doc Instances Table and from `doc_graph.yml` nodes per the semantic naming exception.

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| feature | 001 | Runner Tool | [feature/001_runner_tool.md](feature/001_runner_tool.md) |
| invariant | 001 | Default Flags | [invariant/001_default_flags.md](invariant/001_default_flags.md) |
| invariant | 002 | Dependency Constraints | [invariant/002_dep_constraints.md](invariant/002_dep_constraints.md) |
| api | 001 | Public API | [api/001_public_api.md](api/001_public_api.md) |
