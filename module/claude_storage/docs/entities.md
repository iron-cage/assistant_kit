# Doc Entities

### Scope

- **Purpose**: Index of doc entity directories for `claude_storage`.
- **Responsibility**: Master entity registry for all behavioral requirement doc entities under `docs/`.
- **In Scope**: Behavioral entities (`feature/`, `operation/`, `cli/format/`).
- **Out of Scope**: Test doc entities (→ `tests/doc/entities.md`).

## Entity Tree

```
feature/                         Collection Entity   1st
operation/                       Collection Entity   1st
cli/format/                      Collection Entity   1st
```

## Entities

| Entity | Type | Latent? | Purpose |
|--------|------|---------|---------|
| [feature/](feature/) | Collection | | Feature doc instances covering CLI scope and design decisions |
| [operation/](operation/) | Collection | | Operation doc instances covering upgrade and migration procedures |
| [cli/format/](cli/format/) | Collection | | Format doc instances covering all export rendering modes |

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `feature/` | Index of feature doc instances covering the CLI tool's scope and design decisions. | [feature/readme.md](feature/readme.md) | 1 |
| `operation/` | Index of operation doc instances covering upgrade and migration procedures. | [operation/readme.md](operation/readme.md) | 1 |
| `cli/format/` | Index of format doc instances covering all export rendering modes. | [cli/format/readme.md](cli/format/readme.md) | 3 |

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| feature | 001 | CLI Tool | [feature/001_cli_tool.md](feature/001_cli_tool.md) |
| operation | 001 | Migration Guide | [operation/001_migration_guide.md](operation/001_migration_guide.md) |
| format | markdown | Markdown Export | [cli/format/markdown.md](cli/format/markdown.md) |
| format | json | JSON Export | [cli/format/json.md](cli/format/json.md) |
| format | text | Text Export | [cli/format/text.md](cli/format/text.md) |
