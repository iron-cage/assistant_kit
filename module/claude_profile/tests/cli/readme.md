# tests/cli/

| File | Responsibility |
|------|----------------|
| `cli_runner.rs` | Shared binary runner, assertion, and fixture helpers. |
| `accounts_help_test.rs` | Help output tests (h01–h07). |
| `accounts_list_test.rs` | `.accounts` list command tests acc01–acc25 (Part A). |
| `accounts_list_test_b.rs` | `.accounts` list command tests acc26+ (Part B). |
| `accounts_ft_test.rs` | Feature 037 param unification + mre_324 field alignment (ft01–ft21). |
| `account_mutations_test.rs` | Account save and use tests as01–as22, aw01–aw22 (Part A). |
| `account_mutations_test_b.rs` | Account delete tests ad01+ (Part B). |
| `account_relogin_test.rs` | Account relogin ar01–ar09, aw22–aw25 (Part A). |
| `account_relogin_test_b.rs` | Account relogin aw26–aw30+ lim_it (Part B). |
| `account_renewal_test.rs` | Account renewal arn01–arn21+, save as19–as22 (Part A). |
| `account_renewal_test_b.rs` | Late save tests as23–as35+ (Part B). |
| `account_ownership_test.rs` | Account ownership gates + unclaim tests (ao01–ao11, it01–it11). |
| `account_owner_param_test.rs` | `owner::` parameter EC tests (ft01–ft11, ec01–ec20). |
| `usage_rotate_test.rs` | Feature 038 `rotate::1` on `.usage` — strategy-driven rotation (FT-01–FT-10, EC-05–EC-07). |
| `token_paths_test.rs` | Token status classification and paths output tests. |
| `cross_cutting_test.rs` | Cross-cutting and environment behavior tests. |
| `usage_core_test.rs` | `.usage` core display: heading, JSON, error paths (IT-01–IT-20). |
| `usage_live_test.rs` | `.usage` live mode, streaming, session window tests (IT-21–IT-52). |
| `usage_sort_test.rs` | `.usage` sort, desc, prefer, cols, next:: migration (IT-44–IT-91). |
| `usage_touch_test.rs` | `.usage` touch:: and NextStrategy parameters (IT-92–IT-121). |
| `usage_model_test.rs` | `.usage` imodel::, effort::, structural gates (IT-122–IT-153). |
| `usage_filter_test.rs` | `.usage` row-filtering IT-154–IT-177 (Part A). |
| `usage_filter_test_b.rs` | `.usage` row-filtering IT-178+ (Part B). |
| `usage_lim_it_test.rs` | `.usage` lim_it IT-205–IT-216 (Part A). |
| `usage_lim_it_test_b.rs` | `.usage` lim_it IT-217+ (Part B). |
| `usage_solo_test.rs` | `.usage` solo::, cross-feature corner cases (IT-247–IT-271). |
| `usage_feature_test.rs` | Feature AC coverage tests for `.usage` command (FT-01–FT-05). |
| `persist_test.rs` | PersistPaths: $PRO/$HOME/$USERPROFILE resolution, is_dir guard, ensure_exists. |
| `credentials_test.rs` | `.credentials.status` cred01–cred23 (Part A). |
| `credentials_test_b.rs` | `.credentials.status` cred24+ (Part B). |
| `credentials_status_help_test.rs` | FR-17: `.credentials.status` help descriptions — csh01–csh02. |
| `param_help_test.rs` | Param description presence and optionality guard (BUG-203, BUG-204) — phd01–phd04, pho01–pho04. |
| `account_limits_test.rs` | FR-18: `.account.limits` error paths — lim01–lim05 (IT-5 through IT-8). |
| `dot_test.rs` | Help output and `.` / `.help` delegation tests (dot01–dot12). |
| `account_assign_test.rs` | `.account.assign` marker-only write tests (aa01–aa12). |
| `account_inspect_test.rs` | `.account.inspect` AI-01–AI-17 (Part A). |
| `account_inspect_test_b.rs` | `.account.inspect` AC-18+ lim_it (Part B). |
| `model_test.rs` | Feature 035 `.model` get/set command tests (FT-01..FT-12, IT-01..IT-13, EC-1..EC-6). |
| `set_model_test.rs` | `set_model::` explicit session model override tests (FT-01..FT-09, EC-1..EC-7). |
| `type_test.rs` | CLI type boundary contracts: AccountName (TC-1..6), OutputFormat (TC-1..5), WarningThreshold (TC-1..4), AccountSelector (TC-1..4). |
| `invariant_test.rs` | Architectural invariant assertions: zero-third-party-deps, cross-platform, clear errors, no-process-execution, atomic switching, param defaults (IN-1..2 each). |
| `command_verb_test.rs` | Command-verb behavioral contracts for all 10 verbs: save, use, delete, limits, relogin, rotate, renewal, inspect, assign, status (BV-1..3 each; BV-4 for status). |
| `command_noun_test.rs` | Command-noun contracts for account, token, credentials nouns: lifecycle, JSON output schema, error codes (NC-1..3 each). |
| `user_story_test.rs` | User acceptance tests: account rotation (UA-1..5), onboarding (UA-1..6), quota monitoring (UA-1..5), scripted automation (UA-1..4), credential diagnostics (UA-1..4). |
