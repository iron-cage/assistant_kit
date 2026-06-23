# assistant

Layer 3 super-app aggregating all Layer 2 CLI tools into a single `ast` binary.

## Architecture

```
ast (Layer 3)
  ├── claude_assets   (Layer 2) — 4 commands:  .list, .install, .uninstall, .kinds
  ├── claude_version  (Layer 2) — 12 commands: .status, .version.*, .processes.*, .settings.*, .config
  ├── claude_profile  (Layer 2) — 14 commands: .accounts, .account.*, .credentials.status, .model, .token.status, .paths, .usage
  ├── claude_runner   (Layer 2) — 2 commands:  .claude, .claude.help
  └── claude_storage  (Layer 2) — 9 commands:  .show, .count, .search, .export, .projects, .path, .exists, .session.*
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
| `runbox/` | Container test configuration and runbox entry script |
