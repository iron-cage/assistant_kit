# Test Doc Entities

## Entity Tree

```
cli/testing/                     Composite Entity    1st
├── command/                     Collection Entity   2nd
├── param/                       Collection Entity   2nd
└── param_group/                 Collection Entity   2nd
```

## Entities

| Entity | Type | Latent? | Purpose |
|--------|------|---------|---------|
| [cli/testing/](cli/testing/) | Composite | | CLI testing doc entities: command, param, param group |
| [cli/testing/command/](cli/testing/command/) | Collection | | Per-command integration test case files |
| [cli/testing/param/](cli/testing/param/) | Collection | | Per-parameter edge case test files |
| [cli/testing/param_group/](cli/testing/param_group/) | Collection | | Per-parameter-group interaction test files |

### Scope

- **Purpose**: Index of test doc entity directories for `claude_storage`.
- **Responsibility**: Master entity registry for all test doc entities under `tests/doc/`.
- **In Scope**: CLI testing entities (`cli/testing/command/`, `cli/testing/param/`, `cli/testing/param_group/`).
- **Out of Scope**: Behavioral requirement entities (→ `docs/entities.md`).

### Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `cli/testing/command/` | Index of per-command integration test case files covering command-level behavior. | [cli/testing/command/readme.md](cli/testing/command/readme.md) | 11 |
| `cli/testing/param/` | Index of per-parameter edge case test files covering parameter-level behavior. | [cli/testing/param/readme.md](cli/testing/param/readme.md) | 20 |
| `cli/testing/param_group/` | Index of per-parameter-group interaction test files covering group-level behavior. | [cli/testing/param_group/readme.md](cli/testing/param_group/readme.md) | 5 |

*All entities in this registry use NN-prefixed file names. Individual instances are not enumerated in a Master Doc Instances Table — the Completion Matrix in [`cli/readme.md`](cli/readme.md) tracks coverage instead.*
