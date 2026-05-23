# Test Documentation

Test-lens documentation for `claude_profile`. Each surface subdirectory contains spec files (one per inventory element) with test case indices.

### Responsibility Table

| Directory | Responsibility |
|-----------|----------------|
| cli/ | CLI test case planning: commands, parameters, parameter groups |
| feature/ | Feature behavioral requirement test cases (FT-N entries) |

### Surface Index

| Surface | Source | Case Prefix | Min Cases | Files |
|---------|--------|-------------|-----------|-------|
| CLI commands | `docs/cli/command/` | `IT-` | 8 each | [cli/command/](cli/command/readme.md) |
| CLI parameters | `docs/cli/param/` | `EC-` | 6 each | [cli/param/](cli/param/readme.md) |
| CLI param groups | `docs/cli/param_group/readme.md` | `CC-` | 4 each | [cli/param_group/](cli/param_group/readme.md) |
| Feature docs | `docs/feature/` | `FT-` | 4 each | [feature/](feature/readme.md) |
