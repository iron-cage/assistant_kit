# tests/cli/

| File | Responsibility |
|------|----------------|
| `cli_runner.rs` | Shared binary runner, assertion, and fixture helpers. |
| `accounts_test.rs` | Help output and `.accounts` command tests (h01–h07, acc01–acc16). |
| `account_mutations_test.rs` | Account save, use, and delete tests. |
| `account_rotate_test.rs` | `.account.rotate` redirector tests (DEPRECATED Feature 038; rot01–rot03). |
| `usage_rotate_test.rs` | Feature 038 `rotate::1` on `.usage` — strategy-driven rotation (FT-01–FT-11). |
| `token_paths_test.rs` | Token status classification and paths output tests. |
| `cross_cutting_test.rs` | Cross-cutting and environment behavior tests. |
| `usage_test.rs` | Live rate-limit quota table display tests (IT-1–IT-37, 36 functions). |
| `usage_feature_test.rs` | Feature AC coverage tests for `.usage` command (FT-01–FT-05). |
| `persist_test.rs` | PersistPaths: $PRO/$HOME/$USERPROFILE resolution, is_dir guard, ensure_exists. |
| `credentials_test.rs` | FR-17: `.credentials.status` live cred access without account store — cred01–cred05. |
| `credentials_status_help_test.rs` | FR-17: `.credentials.status` help descriptions — csh01–csh02. |
| `param_help_test.rs` | Param description presence and optionality guard (BUG-203, BUG-204) — phd01–phd04, pho01–pho04. |
| `account_limits_test.rs` | FR-18: `.account.limits` error paths — lim01–lim05 (IT-5 through IT-8). |
| `dot_test.rs` | Help output and `.` / `.help` delegation tests (dot01–dot12). |
| `account_assign_test.rs` | `.account.assign` marker-only write tests (aa01–aa12). |
| `account_inspect_test.rs` | `.account.inspect` command tests. |
| `set_model_test.rs` | `set_model::` explicit session model override tests (FT-01..FT-09, EC-1..EC-7). |
| `type_test.rs` | CLI type boundary contracts: AccountName (TC-1..6), OutputFormat (TC-1..5), WarningThreshold (TC-1..4), AccountSelector (TC-1..4). |
| `invariant_test.rs` | Architectural invariant assertions: zero-third-party-deps, cross-platform, clear errors, no-process-execution, atomic switching, param defaults (IN-1..2 each). |
| `command_verb_test.rs` | Command-verb behavioral contracts for all 10 verbs: save, use, delete, limits, relogin, rotate, renewal, inspect, assign, status (BV-1..3 each; BV-4 for status). |
| `command_noun_test.rs` | Command-noun contracts for account, token, credentials nouns: lifecycle, JSON output schema, error codes (NC-1..3 each). |
| `user_story_test.rs` | User acceptance tests: account rotation (UA-1..5), onboarding (UA-1..6), quota monitoring (UA-1..5), scripted automation (UA-1..4), credential diagnostics (UA-1..4). |
