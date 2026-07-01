//! Integration test crate entry point for `claude_profile`.
//!
//! Includes all integration test modules. Tests invoke the compiled binary
//! via subprocess using `CARGO_BIN_EXE_clp`.
//!
//! ## Coverage Summary
//!
//! This file wires together all CLI integration submodules. Each module covers
//! one functional domain of the `clp` binary:
//!
//! | Module | Domain | Test Series |
//! |--------|--------|-------------|
//! | `accounts_help_test` | help commands | H |
//! | `accounts_list_test` | `.accounts` list command ACC-01–ACC-25 | ACC |
//! | `accounts_list_test_b` | `.accounts` list command ACC-26+ | ACC |
//! | `accounts_ft_test` | Feature 037 param unification + `mre_324` field alignment | FT, mre |
//! | `account_mutations_test` | account save, use Part A | AS, AW |
//! | `account_mutations_test_b` | account delete Part B | AD |
//! | `account_relogin_test` | account relogin + trace/feature027 Part A | AR, AW |
//! | `account_relogin_test_b` | account relogin `lim_it` Part B | AR, AW |
//! | `account_renewal_test` | account renewal + early save tests Part A | ARN, AS |
//! | `account_renewal_test_b` | late save tests Part B | AS |
//! | `account_ownership_test` | account ownership gates + unclaim | AO, AU |
//! | `account_owner_param_test` | `owner::` parameter EC tests | AP |
//! | `token_paths_test` | token status, paths | TS, P |
//! | `cross_cutting_test` | idempotency, param order, exit codes, env | X, E |
//! | `usage_core_test` | .usage core display IT-01–IT-20 | IT |
//! | `usage_live_test` | .usage live mode IT-21–IT-52 | IT |
//! | `usage_sort_test` | .usage sort/desc/prefer/cols IT-44–IT-91 | IT |
//! | `usage_touch_test` | .usage touch/NextStrategy IT-92–IT-121 | IT |
//! | `usage_model_test` | .usage imodel/effort/gates IT-122–IT-153 | IT |
//! | `usage_filter_test` | .usage row-filtering IT-154–IT-177 Part A | IT |
//! | `usage_filter_test_b` | .usage row-filtering IT-178+ Part B | IT |
//! | `usage_lim_it_test` | .usage `lim_it` IT-205–IT-216 Part A | IT |
//! | `usage_lim_it_test_b` | .usage `lim_it` IT-217+ Part B | IT |
//! | `usage_solo_test` | .usage `solo::` + cross-feature IT-247–IT-271 | IT |
//! | `usage_feature_test` | .usage feature AC coverage | FT |
//! | `persist_test` | `PersistPaths` resolution | P |
//! | `credentials_test` | .credentials.status cred01–cred23 Part A | cred |
//! | `credentials_test_b` | .credentials.status cred24+ Part B | cred |
//! | `credentials_status_help_test` | .credentials.status.help descriptions | csh |
//! | `account_limits_test` | .account.limits error paths | lim |
//! | `account_rotate_test` | .account.rotate redirector (DEPRECATED) | ROT |
//! | `usage_rotate_test` | Feature 038 `.usage rotate::1` strategy-driven rotation | FT |
//! | `dot_test` | `.` / `.help` help output | dot |
//! | `param_help_test` | convenience closure param descriptions + optionality | phd, pho |
//! | `account_inspect_test` | .account.inspect AI-01–AI-17 Part A | AI |
//! | `account_inspect_test_b` | .account.inspect AC-18+ Part B | AI |
//! | `account_assign_test` | `.accounts assign::1` marker write | AA |
//! | `set_model_test` | `set_model::` explicit session model override | FT, EC |
//! | `model_test` | `.model` get/set command (Feature 035) | FT |
//! | `type_test` | CLI type boundary contracts (`AccountName`, `OutputFormat`, `WarningThreshold`, `AccountSelector`) | TC |
//! | `invariant_test` | Architectural invariant assertions (zero deps, cross-platform, clear errors, atomic, etc.) | IN |
//! | `command_verb_test` | Command-verb behavioral contracts (save, use, delete, limits, relogin, rotate, renewal, inspect, assign, status) | BV |
//! | `command_noun_test` | Command-noun contracts (account, token, credentials) | NC |
//! | `user_story_test` | User acceptance tests — account rotation, onboarding, quota monitoring, automation, diagnostics | UA |
//!
//! ## Parallel Execution Note
//!
//! These tests spawn the `claude_profile` binary as a subprocess. Under heavy
//! workspace-wide parallel nextest execution, process spawning resource contention
//! can cause intermittent failures (e.g. `x06_param_order_independence_token`).
//! If a test fails here while passing in isolation, run with
//! `cargo nextest run -p claude_profile --no-fail-fast` to distinguish contention
//! from a genuine logic regression. The tests themselves are deterministic.

#[ path = "cli/cli_runner.rs" ]
pub mod cli_runner;

#[ path = "cli/accounts_help_test.rs" ]
mod accounts_help_test;

#[ path = "cli/accounts_list_test.rs" ]
mod accounts_list_test;

#[ path = "cli/accounts_list_test_b.rs" ]
mod accounts_list_test_b;

#[ path = "cli/accounts_ft_test.rs" ]
mod accounts_ft_test;

#[ path = "cli/account_mutations_test.rs" ]
mod account_mutations_test;

#[ path = "cli/account_mutations_test_b.rs" ]
mod account_mutations_test_b;

#[ path = "cli/account_relogin_test.rs" ]
mod account_relogin_test;

#[ path = "cli/account_relogin_test_b.rs" ]
mod account_relogin_test_b;

#[ path = "cli/account_renewal_test.rs" ]
mod account_renewal_test;

#[ path = "cli/account_renewal_test_b.rs" ]
mod account_renewal_test_b;

#[ path = "cli/account_ownership_test.rs" ]
mod account_ownership_test;

#[ path = "cli/account_owner_param_test.rs" ]
mod account_owner_param_test;

#[ path = "cli/token_paths_test.rs" ]
mod token_paths_test;

#[ path = "cli/cross_cutting_test.rs" ]
mod cross_cutting_test;

#[ path = "cli/usage_core_test.rs" ]
mod usage_core_test;

#[ path = "cli/usage_live_test.rs" ]
mod usage_live_test;

#[ path = "cli/usage_sort_test.rs" ]
mod usage_sort_test;

#[ path = "cli/usage_touch_test.rs" ]
mod usage_touch_test;

#[ path = "cli/usage_model_test.rs" ]
mod usage_model_test;

#[ path = "cli/usage_filter_test.rs" ]
mod usage_filter_test;

#[ path = "cli/usage_filter_test_b.rs" ]
mod usage_filter_test_b;

#[ path = "cli/usage_lim_it_test.rs" ]
mod usage_lim_it_test;

#[ path = "cli/usage_lim_it_test_b.rs" ]
mod usage_lim_it_test_b;

#[ path = "cli/usage_solo_test.rs" ]
mod usage_solo_test;

#[ path = "cli/usage_feature_test.rs" ]
mod usage_feature_test;

#[ path = "cli/persist_test.rs" ]
mod persist_test;

#[ path = "cli/credentials_test.rs" ]
mod credentials_test;

#[ path = "cli/credentials_test_b.rs" ]
mod credentials_test_b;

#[ path = "cli/credentials_status_help_test.rs" ]
mod credentials_status_help_test;

#[ path = "cli/account_limits_test.rs" ]
mod account_limits_test;

#[ path = "cli/account_rotate_test.rs" ]
mod account_rotate_test;

#[ path = "cli/usage_rotate_test.rs" ]
mod usage_rotate_test;

#[ path = "cli/dot_test.rs" ]
mod dot_test;

#[ path = "cli/param_help_test.rs" ]
mod param_help_test;

#[ path = "cli/account_inspect_test.rs" ]
mod account_inspect_test;

#[ path = "cli/account_inspect_test_b.rs" ]
mod account_inspect_test_b;

#[ path = "cli/account_assign_test.rs" ]
mod account_assign_test;

#[ path = "cli/set_model_test.rs" ]
mod set_model_test;

#[ path = "cli/model_test.rs" ]
mod model_test;

#[ path = "cli/type_test.rs" ]
mod type_test;

#[ path = "cli/invariant_test.rs" ]
mod invariant_test;

#[ path = "cli/command_verb_test.rs" ]
mod command_verb_test;

#[ path = "cli/command_noun_test.rs" ]
mod command_noun_test;

#[ path = "cli/user_story_test.rs" ]
mod user_story_test;
