# Test Documentation

Test-lens documentation for `claude_profile`. Each surface subdirectory contains spec files (one per inventory element) with test case indices.

### Responsibility Table

| Directory | Responsibility |
|-----------|----------------|
| cli/ | CLI test case planning: commands, parameters, parameter groups |
| feature/ | Feature behavioral requirement test cases (FT-N entries) |
| invariant/ | Invariant property assertion specs (IN-N entries) |
| cli/type/ | CLI type acceptance and boundary spec (TC-N entries) |
| pitfall/ | Pitfall guard verification specs (PP-N entries) |
| algorithm/ | Algorithm correctness specs (AC-N entries) |
| state_machine/ | State machine lifecycle transition specs (AC-N entries) |
| subprocess/ | Subprocess invocation contract specs (AC-N entries) |
| schema/ | Schema constraint specs (SC-N entries) |
| research_interactive/ | Research constraint specs (RC-N entries) |

### Surface Index

| Surface | Source | Case Prefix | Min Cases | Files |
|---------|--------|-------------|-----------|-------|
| CLI commands | `docs/cli/command/` | `IT-` | 8 each | [cli/command/](cli/command/readme.md) |
| CLI parameters | `docs/cli/param/` | `EC-` | 6 each | [cli/param/](cli/param/readme.md) |
| CLI param groups | `docs/cli/param_group/readme.md` | `CC-` | 4 each | [cli/param_group/](cli/param_group/readme.md) |
| Feature docs | `docs/feature/` | `FT-` | 4 each | [feature/](feature/readme.md) |
| Invariant docs | `docs/invariant/` | `IN-` | 2 each | [invariant/](invariant/readme.md) |
| CLI types | `docs/cli/type/` | `TC-` | 4 each | [cli/type/](cli/type/readme.md) |
| Pitfall docs | `docs/pitfall/` | `PP-` | 2 each | [pitfall/](pitfall/readme.md) |
| Algorithm docs | `docs/algorithm/` | `AC-` | 4 each | [algorithm/](algorithm/readme.md) |
| State machine docs | `docs/state_machine/` | `AC-` | 4 each | [state_machine/](state_machine/readme.md) |
| Subprocess docs | `docs/subprocess/` | `AC-` | 4 each | [subprocess/](subprocess/readme.md) |
| CLI user stories | `docs/cli/user_story/` | `UA-` | 4 each | [cli/user_story/](cli/user_story/readme.md) |
| Schema docs | `docs/schema/` | `SC-` | 4 each | [schema/](schema/readme.md) |
| Research interactive docs | `docs/research_interactive/` | `RC-` | 4 each | [research_interactive/](research_interactive/readme.md) |
