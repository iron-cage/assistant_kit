# claude_tools

Layer 3 super-app aggregating all `claude_*` CLI commands into a single `clt` binary.

## Architecture

```
clt (Layer 3)
  ├── claude_manager (Layer 2) — 14 commands: .status, .version.*, .processes.*, .settings.*, .account.*
  ├── claude_profile (Layer 2) — 8 commands:  .account.*, .token.status, .paths, .usage
  ├── claude_runner  (Layer 2) — 2 commands:  .claude, .claude.help
  └── claude_storage (Layer 2) — 9 commands:  .status, .list, .show, .count, .search, .export, .session, .sessions
```

## Usage

```bash
clt .help                     # list all commands
clt .status                   # claude installation status
clt .version.show             # current claude version
clt .version.install          # install/update claude
```

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `src/` | `clt` binary entry and feature-gated Layer 2 registry aggregation |
| `build.rs` | YAML aggregation for `claude_runner` and `claude_storage` commands |
| `tests/` | Compile and link sanity checks for the `clt` binary |
