# assistant

Layer 3 super-app aggregating all Layer 2 CLI tools into a single `ast` binary.

## Architecture

```
ast (Layer 3)
  ├── claude_version (Layer 2) — 14 commands: .status, .version.*, .processes.*, .settings.*, .account.*
  ├── claude_profile (Layer 2) — 8 commands:  .account.*, .token.status, .paths, .usage
  ├── claude_runner  (Layer 2) — 2 commands:  .claude, .claude.help
  └── claude_storage (Layer 2) — 9 commands:  .status, .list, .show, .count, .search, .export, .session, .sessions
```

## Usage

```bash
ast .help                     # list all commands
ast .status                   # claude installation status
ast .version.show             # current claude version
ast .version.install          # install/update claude
```

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `src/` | `ast` binary entry and feature-gated Layer 2 registry aggregation |
| `build.rs` | YAML aggregation for `claude_runner` and `claude_storage` commands |
| `docs/` | Behavioral requirements: feature and invariant doc instances |
| `tests/` | Compile and link sanity checks for the `ast` binary |
| `verb/` | Shell scripts for each `do` protocol verb. |
| `run/` | Shell scripts for container-orchestrated operations. |
