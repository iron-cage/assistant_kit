# Invariant Doc Entity

### Scope

- **Purpose**: Document non-functional constraints that claude_runner must always satisfy.
- **Responsibility**: Index of invariant doc instances covering default flag injection, dependency constraints, command naming convention, trace universality, isolated/refresh subprocess defaults, and exit code contract.
- **In Scope**: Default-on flags (`--dangerously-skip-permissions`, `-c`, `--chrome`), zero consumer workspace dependency rule, binary dependency gating, command naming convention (bare words vs `--` flags), `--trace` universality across all subprocess-executing commands, isolated/refresh subprocess defaults (model, effort, flags, CLAUDE.md, timeout semantics), exit code contract (exit 0/1/2/3/128+N mapping and exit-2 collision disambiguation).
- **Out of Scope**: Feature behavior (→ `feature/`), API contracts (→ `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Default Flags](001_default_flags.md) | Automatic flag injection and opt-out mechanism | ✅ |
| 002 | [Dependency Constraints](002_dep_constraints.md) | Zero consumer workspace deps, binary deps gated by enabled, no routines.rs | ✅ |
| 003 | [Command Naming](003_command_naming.md) | Commands are bare words; parameters use `--`/`-` prefix | ✅ |
| 004 | [Trace Universality](004_trace_universality.md) | Every subprocess-executing command must support `--trace` | ✅ |
| 005 | [Isolated Subprocess Defaults](005_isolated_subprocess_defaults.md) | Model, effort, flags, CLAUDE.md, and timeout semantics for `isolated`/`refresh` | ✅ |
| 006 | [Exit Code Contract](006_exit_codes.md) | Complete exit code table, CLR-layer ad-hoc codes, and exit-2 disambiguation | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |
