# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `feature/` | Version management, process lifecycle, settings, dry-run, CLI design | [feature/readme.md](feature/readme.md) | 5 |
| `pattern/` | 5-layer version lock design | [pattern/readme.md](pattern/readme.md) | 1 |
| `algorithm/` | Settings type inference algorithm | [algorithm/readme.md](algorithm/readme.md) | 1 |
| `cli/testing/command/` | Per-command integration test case indices (semantic naming) | [cli/testing/command/readme.md](cli/testing/command/readme.md) | 12 |
| `cli/testing/param/` | Per-parameter edge case indices (semantic naming) | [cli/testing/param/readme.md](cli/testing/param/readme.md) | 9 |
| `cli/testing/param_group/` | Per-parameter-group integration test indices (semantic naming) | [cli/testing/param_group/readme.md](cli/testing/param_group/readme.md) | 2 |

**Note:** `cli/testing/command/`, `cli/testing/param/`, and `cli/testing/param_group/` use semantic filenames. Their instances are excluded from the Master Doc Instances Table and from `doc_graph.yml` nodes per the semantic naming exception.

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| feature | 001 | Version Management | [feature/001_version_management.md](feature/001_version_management.md) |
| feature | 002 | Process Lifecycle | [feature/002_process_lifecycle.md](feature/002_process_lifecycle.md) |
| feature | 003 | Settings Management | [feature/003_settings_management.md](feature/003_settings_management.md) |
| feature | 004 | Dry Run | [feature/004_dry_run.md](feature/004_dry_run.md) |
| feature | 005 | CLI Design | [feature/005_cli_design.md](feature/005_cli_design.md) |
| pattern | 001 | Version Lock | [pattern/001_version_lock.md](pattern/001_version_lock.md) |
| algorithm | 001 | Settings Type Inference | [algorithm/001_settings_type_inference.md](algorithm/001_settings_type_inference.md) |
