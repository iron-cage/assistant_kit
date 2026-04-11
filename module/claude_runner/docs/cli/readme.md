# clr

Execute Claude Code with configurable `--flag value` parameters.

## Usage

```sh
clr [OPTIONS] [MESSAGE]
```

## Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|----|
| readme.md | done | done | done | done | — | Complete |
| commands.md | done | done | done | done | — | Complete |
| params.md | done | done | done | done | — | Complete |
| dictionary.md | — | done | — | — | — | Complete |
| types.md | — | — | done | done | — | Complete |
| parameter_groups.md | — | — | done | done | — | Complete |
| workflows.md | — | — | done | done | — | Complete |
| parameter_interactions.md | — | — | — | done | — | Complete |
| testing/ | — | — | — | done | done | Complete |

**Current Level:** L5 (Test Detail Complete)

## Navigation

- [Commands](commands.md) — command reference (2 commands)
- [Parameters](params.md) — flag and argument reference (16 parameters)
- [Types](types.md) — semantic type definitions (6 types)
- [Parameter Groups](parameter_groups.md) — logical parameter groupings (3 groups)
- [Dictionary](dictionary.md) — domain vocabulary
- [Workflows](workflows.md) — common usage patterns (9 workflows)
- [Parameter Interactions](parameter_interactions.md) — flag interaction rules and precedence
- [Testing](testing/readme.md) — test case planning (commands, params, groups)

## See Also

- [feature/001_runner_tool.md](../feature/001_runner_tool.md) — architecture, separation of concerns, constraints
- [design_decisions.md](../design_decisions.md) — CLI redesign rationale
