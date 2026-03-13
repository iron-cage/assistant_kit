# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `feature/` | CLI tool functional design and scope | [feature/readme.md](feature/readme.md) | 1 |
| `operation/` | Operational procedures for users and maintainers | [operation/readme.md](operation/readme.md) | 1 |
| `testing/command/` | Per-command test coverage and cases | [cli/testing/command/readme.md](cli/testing/command/readme.md) | 13 |
| `testing/param/` | Per-parameter test coverage and cases | [cli/testing/param/readme.md](cli/testing/param/readme.md) | 20 |
| `testing/param_group/` | Per-parameter-group test coverage and cases | [cli/testing/param_group/readme.md](cli/testing/param_group/readme.md) | 5 |

**Note:** `testing/command/`, `testing/param/`, and `testing/param_group/` use semantic filenames. Their instances are excluded from the Master Doc Instances Table and from `doc_graph.yml` nodes per the semantic naming exception.

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| feature | 001 | CLI Tool | [feature/001_cli_tool.md](feature/001_cli_tool.md) |
| operation | 001 | Migration Guide | [operation/001_migration_guide.md](operation/001_migration_guide.md) |
