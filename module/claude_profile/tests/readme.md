# tests/

| File | Responsibility |
|------|----------------|
| `account_tests.rs` | Account CRUD: save, list, switch, delete, active-guard. |
| `token_tests.rs` | TokenStatus classification: Valid, ExpiringSoon, Expired. |
| `paths_tests.rs` | ClaudePaths: all canonical path methods, HOME-not-set guard. |
| `responsibility_no_process_execution_test.rs` | Guard: no std::process import anywhere in crate source. |
| `lib_test.rs` | Library exports: COMMANDS_YAML, register_commands(), command presence. |
| `cli_adapter_test.rs` | Adapter and output module: argv conversion, aliases, bool normalization, validation, json_escape, format_duration_secs. |
| `cli_clp_alias_test.rs` | Binary alias smoke tests: both `clp` and `claude_profile` aliases run and self-identify. |
| `cli_integration_test.rs` | CLI binary integration: entry point for all cli/ submodules. |
| `usage_integration_test.rs` | Entry point wiring all tests/usage/ integration test modules. |
| `cli/` | Split integration test modules (help, accounts-list, accounts-ft, mutations, relogin, renewal, ownership, owner-param, rotate, token, paths, usage-core, usage-live, usage-sort, usage-touch, usage-model, usage-filter, usage-lim_it, usage-solo, usage-feature, usage-rotate, persist, credentials, limits, param-help, dot, cross-cutting, account-inspect, set-model, type-contracts, invariants, command-verbs, command-nouns, user-stories). |
| `usage/` | Integration tests for src/usage/ internals via test_bridge (format, render, sort_next, touch, refresh, fetch, api). |
| `manual/` | Manual testing plan: live Claude Code account switching. |
| `docs/` | Test-lens documentation: per-command, per-parameter, per-group, and per-feature test case indices. |

## Scope

### Responsibilities

These tests cover the `claude_profile` crate: account credential management,
token status classification, canonical path resolution, and the `clp` CLI binary.

### In Scope

- Library unit tests (account, token, paths): real tmpdir HOME, no subprocess
- CLI integration tests (`cli/`): subprocess invocation of compiled `clp` binary
- Binary alias smoke tests (`cli_clp_alias_test.rs`): both `clp` and `claude_profile` aliases
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
├── cli_clp_alias_test.rs                 # binary alias smoke tests (clp + claude_profile)
├── cli_integration_test.rs               # integration test entry point
├── usage_integration_test.rs             # usage integration test entry point
├── responsibility_no_process_execution_test.rs  # arch boundary guard
├── cli/
│   ├── readme.md                         # integration submodule index
│   ├── cli_runner.rs                     # shared binary runner + fixtures
│   ├── accounts_help_test.rs             # help output tests (h01–h07)
│   ├── accounts_list_test.rs             # .accounts list command tests (acc01–acc50 + mre_324)
│   ├── accounts_ft_test.rs               # Feature 037 param unification + mre_324 field alignment (ft01–ft21)
│   ├── account_mutations_test.rs         # account save, use, delete (as01–as18, aw01–aw17, ad01–ad15)
│   ├── account_relogin_test.rs           # account relogin + AW trace/feature027 (ar01–ar09, aw22–aw35)
│   ├── account_renewal_test.rs           # account renewal + late save tests (arn01–arn21, as19–as35)
│   ├── account_ownership_test.rs         # account ownership gates + unclaim (ao01–ao11, it01–it11)
│   ├── account_owner_param_test.rs       # owner:: parameter EC tests (ft01–ft11, ec01–ec20)
│   ├── account_rotate_test.rs            # .account.rotate deprecated redirector (rot01–rot03)
│   ├── token_paths_test.rs               # token status + paths commands
│   ├── cross_cutting_test.rs             # idempotency, param order, exit codes
│   ├── usage_core_test.rs                # .usage core display: heading, JSON, error paths (IT-01–IT-20)
│   ├── usage_live_test.rs                # .usage live mode, streaming, session window (IT-21–IT-52)
│   ├── usage_sort_test.rs                # .usage sort, desc, prefer, cols, next:: migration (IT-44–IT-91)
│   ├── usage_touch_test.rs               # .usage touch:: and NextStrategy parameters (IT-92–IT-121)
│   ├── usage_model_test.rs               # .usage imodel::, effort::, structural gates (IT-122–IT-153)
│   ├── usage_filter_test.rs              # .usage row-filtering, get::, abs::, no_color:: (IT-154–IT-205)
│   ├── usage_lim_it_test.rs              # .usage live lim_it filter, get::, format tests (IT-205–IT-247)
│   ├── usage_solo_test.rs                # .usage solo::, cross-feature corner cases (IT-247–IT-271)
│   ├── usage_feature_test.rs             # .usage feature AC coverage (FT-01–FT-05)
│   ├── usage_rotate_test.rs              # .usage rotate::1 strategy-driven rotation (FT-01–FT-10, EC-05–EC-07)
│   ├── persist_test.rs                   # PersistPaths resolution tests
│   ├── credentials_test.rs               # .credentials.status command tests
│   ├── credentials_status_help_test.rs   # .credentials.status help descriptions
│   ├── param_help_test.rs                # param description presence + optionality (BUG-203, BUG-204)
│   ├── account_limits_test.rs            # .account.limits error paths
│   ├── dot_test.rs                       # . / .help output tests
│   ├── account_assign_test.rs            # .account.assign marker-only write tests
│   ├── account_inspect_test.rs           # .account.inspect command tests
│   ├── set_model_test.rs                 # explicit session model override (FT-01..FT-09, EC-1..EC-7)
│   ├── type_test.rs                      # CLI type boundary contracts (AccountName, OutputFormat, etc.)
│   ├── invariant_test.rs                 # architectural invariant assertions (IN-1..2 each)
│   ├── command_verb_test.rs              # command-verb behavioral contracts (10 verbs, BV-1..4)
│   ├── command_noun_test.rs              # command-noun contracts (account, token, credentials)
│   └── user_story_test.rs               # user acceptance tests (UA scenarios)
├── manual/
│   └── readme.md                         # manual testing plan
└── docs/
    ├── readme.md                         # test docs surface index
    ├── cli/
    │   ├── readme.md                     # CLI test-lens index
    │   ├── command/                      # per-command test case files (000–016)
    │   ├── param/                        # per-parameter test case files (01–60)
    │   └── param_group/                  # per-group test case files (001–006)
    └── feature/
        ├── readme.md                     # feature test-lens index
        └── [38 spec files]               # FT cases for Features 001–038 (full index in readme.md)
```

## Domain Map

| Domain | Test Location | What It Tests |
|--------|---------------|---------------|
| Account CRUD (library) | `account_tests.rs` | save, list, switch, delete, helpers |
| Token classification (library) | `token_tests.rs` | status, status_with_threshold, parse_expires_at |
| Path resolution (library) | `paths_tests.rs` | ClaudePaths construction and all path methods |
| Library exports | `lib_test.rs` | COMMANDS_YAML, register_commands, command presence |
| Adapter + output | `cli_adapter_test.rs` | argv_to_unilang_tokens, OutputOptions, json_escape, format_duration_secs |
| Binary alias smoke | `cli_clp_alias_test.rs` | `clp` and `claude_profile` aliases run and self-identify |
| Help CLI | `cli/accounts_help_test.rs` | --help, .help, no-args, unknown command (h01–h07) |
| Accounts CLI | `cli/accounts_list_test.rs` | list text/json, empty dir, sorted, field-presence (acc01–acc50, mre_324) |
| Accounts feature unification CLI | `cli/accounts_ft_test.rs` | Feature 037 param unification + mre_324 field alignment (ft01–ft21) |
| Account save/use/delete CLI | `cli/account_mutations_test.rs` | save, use, delete (as01–as18, aw01–aw17, ad01–ad15) |
| Account relogin CLI | `cli/account_relogin_test.rs` | relogin + AW trace/feature027 (ar01–ar09, aw22–aw35) |
| Account renewal CLI | `cli/account_renewal_test.rs` | renewal + late save (arn01–arn21, as19–as35) |
| Account ownership CLI | `cli/account_ownership_test.rs` | ownership gates + unclaim (ao01–ao11, it01–it11) |
| Account owner:: param CLI | `cli/account_owner_param_test.rs` | owner:: EC tests (ft01–ft11, ec01–ec20) |
| Account rotate CLI | `cli/account_rotate_test.rs` | .account.rotate deprecated redirector, rot01–rot03 |
| Token status + paths CLI | `cli/token_paths_test.rs` | .token.status and .paths all verbosity/format |
| Cross-cutting CLI | `cli/cross_cutting_test.rs` | idempotency, param order, exit code contracts, env |
| Usage core CLI | `cli/usage_core_test.rs` | .usage heading, JSON, error paths (IT-01–IT-20) |
| Usage live CLI | `cli/usage_live_test.rs` | .usage live mode, streaming, session window (IT-21–IT-52) |
| Usage sort CLI | `cli/usage_sort_test.rs` | .usage sort, desc, prefer, cols, next:: (IT-44–IT-91) |
| Usage touch CLI | `cli/usage_touch_test.rs` | .usage touch:: and NextStrategy (IT-92–IT-121) |
| Usage model CLI | `cli/usage_model_test.rs` | .usage imodel::, effort::, structural gates (IT-122–IT-153) |
| Usage filter CLI | `cli/usage_filter_test.rs` | .usage row-filtering, get::, abs::, no_color:: (IT-154–IT-205) |
| Usage lim_it CLI | `cli/usage_lim_it_test.rs` | .usage lim_it filter, get::, format (IT-205–IT-247) |
| Usage solo CLI | `cli/usage_solo_test.rs` | .usage solo::, cross-feature corner cases (IT-247–IT-271) |
| Usage feature AC | `cli/usage_feature_test.rs` | .usage acceptance criteria (AC-01–AC-06) |
| Usage rotate CLI | `cli/usage_rotate_test.rs` | .usage rotate::1 strategy-driven rotation, FT-01–FT-10, EC-05–EC-07 |
| Persist paths | `cli/persist_test.rs` | PersistPaths PRO/HOME resolution, ensure_exists |
| Credentials status CLI | `cli/credentials_test.rs` | .credentials.status without account store |
| Credentials status help CLI | `cli/credentials_status_help_test.rs` | .credentials.status help descriptions |
| Account limits CLI | `cli/account_limits_test.rs` | .account.limits error paths |
| Param help/optionality CLI | `cli/param_help_test.rs` | phd01–phd04 (BUG-203), pho01–pho04 (BUG-204) |
| Dot / help CLI | `cli/dot_test.rs` | . and .help output, delegation, ANSI suppression |
| Account assign CLI | `cli/account_assign_test.rs` | .account.assign marker-only write (aa01–aa12) |
| Account inspect CLI | `cli/account_inspect_test.rs` | .account.inspect command tests |
| Session model override CLI | `cli/set_model_test.rs` | set_model:: explicit session model override (FT-01..FT-09, EC-1..EC-7) |
| Type boundary contracts | `cli/type_test.rs` | AccountName, OutputFormat, WarningThreshold, AccountSelector contracts |
| Architectural invariants | `cli/invariant_test.rs` | zero-third-party-deps, cross-platform, atomic switching, param defaults |
| Command-verb contracts | `cli/command_verb_test.rs` | behavioral contracts for 10 command verbs (BV-1..4 each) |
| Command-noun contracts | `cli/command_noun_test.rs` | account, token, credentials noun contracts (NC-1..3) |
| User acceptance | `cli/user_story_test.rs` | account rotation, onboarding, quota monitoring, scripted automation (UA scenarios) |
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
