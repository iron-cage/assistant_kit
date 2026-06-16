# Environment Parameters

### Scope

- **Purpose**: Environment variable reference for the clv CLI.
- **Responsibility**: All environment variables that affect clv behavior — names, types, defaults, and consuming commands.
- **In Scope**: All env vars read by clv at startup or during command execution.
- **Out of Scope**: CLI parameter reference (→ `param/`), config file parameters (→ `config_param.md`).

### All Environment Variables

| # | Variable | Type | Default | Purpose |
|---|----------|------|---------|---------|
| 1 | `HOME` | Path | *(OS-provided)* | Locates settings file and credential store |
| 2 | `CLAUDE_MODEL` | String | — | Overrides `model` setting; highest priority in `.config` resolution chain |

---

### Variable :: 1. `HOME`

Standard Unix home directory path. clv uses this to resolve:
- `~/.claude/settings.json` — settings read/write target
- `~/.persistent/claude/credential/_active` — active account marker (or `$HOME/.persistent/...`)

If `HOME` is unset, commands that access settings or credentials exit with code 2.

**Consumed by:** `.status`, `.version.install`, `.version.guard`, `.version.history`, `.settings.show`, `.settings.get`, `.settings.set`, `.config`

---

### Variable :: 2. `CLAUDE_MODEL`

When set, provides the effective value for the `model` settings key in the `.config` resolution chain (env layer = highest priority). Overrides project config, user config, and catalog default.

If set to an empty string, it is treated as absent (env layer skipped for `model`).

**Consumed by:** `.config` (env layer of resolution chain, key `model` only)
