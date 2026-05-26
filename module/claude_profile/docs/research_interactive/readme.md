# research_interactive

### Scope

- **Purpose**: Investigation findings on Claude binary behavior and interaction modes.
- **Responsibility**: Research notes derived from `claude --help`, live experiments, and source analysis — informing design decisions for commands that spawn claude subprocesses.
- **In Scope**: Binary flag inventory, auth subcommand analysis, process execution modes, credential write behavior, environment variable controls.
- **Out of Scope**: Behavioral requirements (→ `feature/`), test specs (→ `tests/docs/`).

| File | Responsibility |
|------|----------------|
| 001_claude_interactive_session_control.md | Findings on controlling/avoiding full interactive REPL — `claude auth login` discovery |
