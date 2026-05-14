# tests/

| File | Responsibility |
|------|----------------|
| `account_tests.rs` | Account CRUD: save, list, switch, delete, active-guard. |
| `token_tests.rs` | TokenStatus classification: Valid, ExpiringSoon, Expired. |
| `paths_tests.rs` | ClaudePaths: all canonical path methods, HOME-not-set guard. |
| `responsibility_no_process_execution_test.rs` | Guard: no std::process import anywhere in crate source. |
| `lib_test.rs` | Library exports: COMMANDS_YAML, register_commands(), command presence. |
| `cli_adapter_test.rs` | Adapter and output module: argv conversion, aliases, bool normalization, validation, json_escape, format_duration_secs. |
| `cli_integration_test.rs` | CLI binary integration: entry point for cli/ modules. |
| `cli/` | Split integration test modules (help, accounts, mutations, token, paths, usage, persist, credentials, limits, dot, cross-cutting). |
| `manual/` | Manual testing plan: live Claude Code account switching. |
| `docs/` | Test-lens documentation: per-command, per-parameter, and per-group test case indices. |

## Scope

### Responsibilities

These tests cover the `claude_profile` crate: account credential management,
token status classification, canonical path resolution, and the `clp` CLI binary.

### In Scope

- Library unit tests (account, token, paths): real tmpdir HOME, no subprocess
- CLI integration tests (`cli/`): subprocess invocation of compiled `clp` binary
- Library-level export verification (`lib_test.rs`)
- Adapter and output module logic (`cli_adapter_test.rs`)
- Responsibility boundary guards (no `std::process::Command` in crate source)

### Out of Scope

- Tests for `claude_profile_core` — those belong in `claude_profile_core/tests/`
- Tests for other crates (`claude_runner`, `claude_storage`, etc.)
- Performance benchmarks — belong in `benches/`

## Organization Principles

Tests are organized by functional domain (what is tested), not methodology.
Top-level test files cover discrete library domains (account, token, paths).
CLI end-to-end tests are split into focused domain files under `cli/`
and loaded through the `cli_integration_test.rs` entry point.

## Directory Structure

```text
tests/
├── readme.md                             # this file
├── account_tests.rs                      # account CRUD library tests
├── token_tests.rs                        # token classification library tests
├── paths_tests.rs                        # ClaudePaths library tests
├── lib_test.rs                           # library export smoke tests
├── cli_adapter_test.rs                   # adapter + output unit tests
├── cli_integration_test.rs               # integration test entry point
├── responsibility_no_process_execution_test.rs  # arch boundary guard
├── cli/
│   ├── readme.md                         # integration submodule index
│   ├── helpers.rs                        # shared binary runner + fixtures
│   ├── accounts_test.rs                  # help output and .accounts command
│   ├── account_mutations_test.rs         # account save, use, delete
│   ├── token_paths_test.rs               # token status + paths commands
│   ├── cross_cutting_test.rs             # idempotency, param order, exit codes
│   ├── usage_test.rs                     # .usage command tests
│   ├── persist_test.rs                   # PersistPaths resolution tests
│   ├── credentials_test.rs               # .credentials.status command tests
│   ├── credentials_status_help_test.rs   # .credentials.status help descriptions
│   ├── account_limits_test.rs            # .account.limits error paths
│   └── dot_test.rs                       # . / .help output tests
├── manual/
│   └── readme.md                         # manual testing plan
└── docs/
    └── cli/
        ├── readme.md                     # CLI test-lens index
        ├── command/                      # per-command test case files (00–11)
        ├── param/                        # per-parameter test case files (01–05)
        └── param_group/                  # per-group test case files (01–02)
```

## Domain Map

| Domain | Test Location | What It Tests |
|--------|---------------|---------------|
| Account CRUD (library) | `account_tests.rs` | save, list, switch, delete, auto_rotate, helpers |
| Token classification (library) | `token_tests.rs` | status, status_with_threshold, parse_expires_at |
| Path resolution (library) | `paths_tests.rs` | ClaudePaths construction and all path methods |
| Library exports | `lib_test.rs` | COMMANDS_YAML, register_commands, command presence |
| Adapter + output | `cli_adapter_test.rs` | argv_to_unilang_tokens, OutputOptions, json_escape, format_duration_secs |
| Help CLI | `cli/accounts_test.rs` (H series) | --help, .help, no-args, unknown command |
| Accounts CLI | `cli/accounts_test.rs` (acc series) | list text/json, empty dir, sorted, field-presence, named-account |
| Account save/use/delete CLI | `cli/account_mutations_test.rs` | save, use, delete with all edge cases |
| Token status + paths CLI | `cli/token_paths_test.rs` | .token.status and .paths all verbosity/format |
| Cross-cutting CLI | `cli/cross_cutting_test.rs` | idempotency, param order, exit code contracts, env |
| Usage CLI | `cli/usage_test.rs` | .usage live quota table, JSON output, error paths |
| Persist paths | `cli/persist_test.rs` | PersistPaths PRO/HOME resolution, ensure_exists |
| Credentials status CLI | `cli/credentials_test.rs` | .credentials.status without account store |
| Credentials status help CLI | `cli/credentials_status_help_test.rs` | .credentials.status help descriptions |
| Account limits CLI | `cli/account_limits_test.rs` | .account.limits error paths |
| Dot / help CLI | `cli/dot_test.rs` | . and .help output, delegation, ANSI suppression |
| Arch boundary | `responsibility_no_process_execution_test.rs` | no std::process in crate source |

## Adding New Tests

**Q: Testing a new library function in `src/account.rs`?**
→ Add to `account_tests.rs` (account domain). Update test matrix in that file.

**Q: Testing a new CLI command end-to-end?**
→ Create or extend a file in `cli/` matching the command's domain.
→ Wire it into `cli_integration_test.rs` with a new `mod` block.
→ Update `cli/readme.md`.

**Q: Testing a new library module (e.g., `src/foo.rs`)?**
→ Create `tests/foo_tests.rs`. Add a row to this readme's Responsibility Table and Domain Map.

**Q: Testing an invariant across the whole crate?**
→ Add to `responsibility_no_process_execution_test.rs` if it is an arch boundary guard,
or create a dedicated `tests/<invariant>_test.rs` file.
