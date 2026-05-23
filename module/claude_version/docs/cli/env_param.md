# Environment Parameters

### Scope

- **Purpose**: Environment variable reference for the cm CLI.
- **Responsibility**: All environment variables that affect cm behavior — names, types, defaults, and consuming commands.
- **In Scope**: All env vars read by cm at startup or during command execution.
- **Out of Scope**: CLI parameter reference (→ `005_params.md`), config file parameters (→ `config_param.md`).

### All Environment Variables (1 total)

| # | Variable | Type | Default | Purpose |
|---|----------|------|---------|---------|
| 1 | `HOME` | Path | *(OS-provided)* | Locates settings file and credential store |

---

### Variable :: 1. `HOME`

Standard Unix home directory path. cm uses this to resolve:
- `~/.claude/settings.json` — settings read/write target
- `~/.persistent/claude/credential/_active` — active account marker (or `$HOME/.persistent/...`)

If `HOME` is unset, commands that access settings or credentials exit with code 2.

**Consumed by:** `.status`, `.version.install`, `.version.guard`, `.version.history`, `.settings.show`, `.settings.get`, `.settings.set`
