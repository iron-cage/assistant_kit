# command/ — clvCommand Reference

### Scope

- **Purpose**: Per-namespace command reference for all 13 clvcommands.
- **Responsibility**: Command syntax, parameters, exit codes, examples, and cross-references grouped by namespace.
- **In Scope**: All 13 clvcommands, organized by dot-namespace cluster.
- **Out of Scope**: Parameter details (→ `../param/`), type definitions (→ `../type/`), behavioral contracts (→ `../../feature/`).

### Responsibility Table

| File | Responsibility |
|------|---------------|
| readme.md | Index and navigation for command namespace files |
| procedure.md | Steps for adding, updating, or removing command instances |
| root.md | Root-namespace commands: `.help`, `.status` |
| version.md | Version-namespace commands: `.version.*` (5 commands) |
| processes.md | Process-namespace commands: `.processes`, `.processes.kill` |
| settings.md | Settings-namespace commands: `.settings.*` (3 commands, deprecated) |
| config.md | Config command: `.config` (unified settings inspection and modification) |

### All Commands (13 total)

| # | Command | Namespace | File |
|---|---------|-----------|------|
| 1 | `.help` | root | [root.md](root.md) |
| 2 | `.status` | root | [root.md](root.md) |
| 3 | `.version.show` | version | [version.md](version.md) |
| 4 | `.version.install` | version | [version.md](version.md) |
| 5 | `.version.guard` | version | [version.md](version.md) |
| 6 | `.version.list` | version | [version.md](version.md) |
| 7 | `.processes` | processes | [processes.md](processes.md) |
| 8 | `.processes.kill` | processes | [processes.md](processes.md) |
| 9 | `.settings.show` | settings | [settings.md](settings.md) *(deprecated)* |
| 10 | `.settings.get` | settings | [settings.md](settings.md) *(deprecated)* |
| 11 | `.settings.set` | settings | [settings.md](settings.md) *(deprecated)* |
| 12 | `.version.history` | version | [version.md](version.md) |
| 13 | `.config` | config | [config.md](config.md) |

### Navigation

- [Root Commands](root.md) — `.help`, `.status`
- [Version Commands](version.md) — `.version.show`, `.version.install`, `.version.guard`, `.version.list`, `.version.history`
- [Process Commands](processes.md) — `.processes`, `.processes.kill`
- [Settings Commands](settings.md) — `.settings.show`, `.settings.get`, `.settings.set` *(deprecated)*
- [Config Command](config.md) — `.config`

### See Also

- [Parameters](../param/readme.md) — flag reference
- [Types](../type/readme.md) — semantic type definitions
- [Parameter Groups](../param_group/readme.md) — logical parameter groupings
- [User Stories](../user_story/readme.md) — persona-goal scenarios
