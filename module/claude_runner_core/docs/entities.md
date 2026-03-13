# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `claude_params/` | Per-parameter reference for all claude binary flags (semantic naming) | [claude_params/readme.md](claude_params/readme.md) | 60+ |
| `pattern/` | Builder pattern design and rationale | [pattern/readme.md](pattern/readme.md) | 1 |
| `api/` | Execution API contracts and method signatures | [api/readme.md](api/readme.md) | 1 |
| `data_structure/` | Type-safe configuration enum definitions | [data_structure/readme.md](data_structure/readme.md) | 1 |
| `feature/` | Execution control, dry-run, and describe features | [feature/readme.md](feature/readme.md) | 3 |
| `invariant/` | Single execution point and NFR conformance constraints | [invariant/readme.md](invariant/readme.md) | 2 |

**Note:** `claude_params/` uses semantic filenames (one file per claude binary parameter, e.g. `dry_run.md`, `output_format.md`). Its instances are excluded from the Master Doc Instances Table and from `doc_graph.yml` nodes per the semantic naming exception.

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| pattern | 001 | Command Builder | [pattern/001_command_builder.md](pattern/001_command_builder.md) |
| api | 001 | Execution API | [api/001_execution_api.md](api/001_execution_api.md) |
| data_structure | 001 | Command Types | [data_structure/001_command_types.md](data_structure/001_command_types.md) |
| feature | 001 | Execution Control | [feature/001_execution_control.md](feature/001_execution_control.md) |
| feature | 002 | Dry Run | [feature/002_dry_run.md](feature/002_dry_run.md) |
| feature | 003 | Describe | [feature/003_describe.md](feature/003_describe.md) |
| invariant | 001 | Single Execution Point | [invariant/001_single_execution_point.md](invariant/001_single_execution_point.md) |
| invariant | 002 | NFR Conformance | [invariant/002_nfr_conformance.md](invariant/002_nfr_conformance.md) |
