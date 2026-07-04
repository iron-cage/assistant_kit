# Research Interactive Doc Entity

RC-N research constraint specs for `claude_profile`. Covers testable constraints and
implementation properties discovered via research into `claude` binary execution modes and
authentication subcommands, as documented in `docs/research_interactive/`.

**RC- extension note:** RC- (Research Constraint) is a project-local element type
extension. `docs/research_interactive/` maps to `tests/docs/research_interactive/` as
a research documentation surface. Cases document verifiable system properties derived
from research findings. Min 4 RC- cases per spec.

### Responsibility Table

| File | Research Doc | RC-N Cases |
|------|-------------|-----------|
| `001_claude_interactive_session_control.md` | Claude binary execution modes and auth subcommands | RC-1 through RC-4 |

### Coverage Summary

| Research Files | Total RC- Cases |
|---------------|-----------------|
| 1 | 4 |

### See Also

- [docs/research_interactive/001_claude_interactive_session_control.md](../../../docs/research_interactive/001_claude_interactive_session_control.md) — research source doc
- [tests/docs/cli/command/12_account_relogin.md](../cli/command/12_account_relogin.md) — relogin command integration tests
