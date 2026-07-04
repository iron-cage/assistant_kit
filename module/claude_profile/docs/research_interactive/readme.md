# Research Interactive Doc Entity

### Scope

- **Purpose**: Investigation findings on Claude binary behavior and interaction modes.
- **Responsibility**: Research notes derived from `claude --help`, live experiments, and source analysis — informing design decisions for commands that spawn claude subprocesses.
- **In Scope**: Binary flag inventory, auth subcommand analysis, process execution modes, credential write behavior, environment variable controls.
- **Out of Scope**: Behavioral requirements (→ `feature/`), test specs (→ `tests/docs/`).

### Type Declaration

- **Type name**: Research Interactive
- **Extends**: Doc Entity (local extension — not a built-in type in `doc_des.rulebook.md`)
- **Instance naming**: `{NNN}_{investigation_topic}.md` (NNN = 3-digit ID)
- **Required instance sections**: `### Scope` (4 bullets), `### Findings`
- **Optional instance sections**: Typed reference sections (`### Features`, `### Subprocess`)

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| — | [procedure](procedure.md) | Workflow for capturing and updating research findings | ✅ |
| 001 | [claude_interactive_session_control](001_claude_interactive_session_control.md) | Findings on `claude auth login` discovery and execution modes | ✅ |
