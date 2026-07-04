# Variable: CLAUDE_HOME

### Scope

- **Purpose**: Document the base Claude configuration directory — the root under which all Claude Code session and memory storage lives.
- **Responsibility**: Define the value format, derivation rule, override mechanism, and examples for `CLAUDE_HOME`.
- **In Scope**: CLAUDE_HOME default, env var override, relation to CLAUDE_PROJECTS_DIR.
- **Out of Scope**: Downstream variables (→ `002_claude_projects_dir.md` through `006_claude_session_file.md`); algorithm details (→ `../algorithm/`).

### Value Format

Absolute filesystem path to a directory. Never ends with `/`.

**Examples:** `/home/alice/.claude`, `/opt/claude-config`, `/root/.claude`

### Derivation

```
CLAUDE_HOME = ${CLAUDE_HOME:-$HOME/.claude}
```

1. If the `CLAUDE_HOME` environment variable is set: use its value directly (an empty value is used as-is; the implementation does not special-case it).
2. Otherwise: use `$HOME/.claude`.

`HOME` is the standard POSIX home directory env var. On Linux/macOS, `HOME` is always set for interactive users. In containerized environments, `HOME` may need to be set explicitly.

### Override

| Mechanism | Value |
|-----------|-------|
| Env var | `CLAUDE_HOME` — set to any absolute path to relocate the entire Claude config tree |
| Default | `$HOME/.claude` |

**Usage:**

```sh
# Use a custom config directory
CLAUDE_HOME=/opt/claude-config clr scope

# Inspect the effective CLAUDE_HOME
clr scope --dir /home/alice/project
# Output includes: CLAUDE_HOME=/home/alice/.claude
```

### Examples

| Scenario | CLAUDE_HOME |
|----------|-------------|
| Default on Linux | `/home/alice/.claude` |
| Default in container (HOME=/root) | `/root/.claude` |
| Override via env var | `CLAUDE_HOME=/opt/claude-config` → `/opt/claude-config` |
| clr isolated (temp HOME) | `<temp_dir>/.claude` (internal; not exposed via `clr scope`) |

### Related Docs

| File | Relationship |
|------|--------------|
| [`002_claude_projects_dir.md`](002_claude_projects_dir.md) | CLAUDE_PROJECTS_DIR = CLAUDE_HOME/projects/ |
| [`../algorithm/001_path_encoding.md`](../algorithm/001_path_encoding.md) | Df() — applied after CLAUDE_PROJECTS_DIR is resolved |
| [`../feature/005_session_path_resolution.md`](../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` and session cross-loading |
