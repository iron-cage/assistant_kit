# View complete usage information without executing Claude

**Persona:** New user evaluating the `clr` CLI who needs to discover available commands, parameters, and usage patterns before running any task.
**Goal:** View a complete usage summary — commands, flags, and invocation syntax — without executing any Claude subprocess or modifying any state.
**Benefit:** Enables self-service onboarding without consulting external documentation.
**Priority:** Low

### Acceptance Criteria

- `clr help` prints usage information and exits with code 0
- `clr -h` and `clr --help` produce identical output to `clr help`
- Help output lists all 8 available subcommands (run, ask, isolated, refresh, ps, kill, tools, help) in two structured sections: RUNNER OPTIONS and CLAUDE CODE OPTIONS (forwarded)
- Help output lists available flags with short descriptions
- No Claude subprocess is launched; no credentials are required
- No session state is read or written

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 4 | [`help`](../command/04_help.md) | Prints usage information and exits |

### Referenced Parameter Groups

*None applicable — `help` accepts no parameters and belongs to no group.*

### Referenced Parameters

*None — `help` is a zero-parameter command.*

### Workflow Steps

1. `clr help` — print full usage information and exit with code 0
2. `clr --help` — identical output via long flag
3. `clr -h` — identical output via short flag
