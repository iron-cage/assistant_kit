# JSON Config Loading — Implementation

## Execution State

- **Executor Type:** any
- **filed_by:** agent
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Done)
- **closes:** null
- **dir:** src/
- **validated_by:** agent
- **validation_date:** 2026-07-02

## Goal

Implement the JSON config loading feature documented in `feature/004_json_config.md`: add `--args-file <PATH>` flag and `CLR_ARGS_FILE` env var to all four executing subcommands (`run`, `ask`, `isolated`, `refresh`); implement stdin JSON pipe auto-detection; apply JSON-sourced parameters between CLI flag parsing and CLR_* env var fallbacks (CLI > JSON > CLR_* > defaults).

Observable end-state: `clr --args-file fast.json --dry-run "task"` prints a command preview showing parameters from `fast.json`; `echo '{"max-sessions":0}' | clr --dry-run "task"` auto-detects stdin JSON and applies `max-sessions 0`; `CLR_ARGS_FILE=/path.json clr "task"` is equivalent to `--args-file /path.json`; a missing file exits 1 with a file-not-found error on stderr before subprocess spawn; `w3 .test level::3` passes with zero failures.

## In Scope

- `src/cli/parse.rs` — add `args_file: Option<String>` field to `CliArgs`; add `--args-file <PATH>` token parsing in `parse_args()`
- `src/cli/env.rs` — sole owner of `CLR_ARGS_FILE` reading: parse it in `apply_env_vars()` and set `args_file` field only when not already set by a CLI `--args-file` token
- `src/cli/mod.rs` — two distinct responsibilities in `run_cli()`: (a) stdin JSON detection (early, before any parse): check `!IsTerminal::is_terminal(&stdin())` and first byte `{`; for `run`/`ask` subcommands skip when `--file` appears in raw tokens (raw scan before full parse); for `isolated`/`refresh` subcommands `--file` is unsupported so stdin detection is always active when stdin is JSON; (b) JSON token injection (before `parse_args()`): load JSON config from resolved `args_file` or `CLR_ARGS_FILE`; append JSON-derived tokens AFTER explicit CLI tokens so first-occurrence-wins gives CLI priority
- New `src/cli/json_config.rs` — JSON loading and key-to-CliArgs mapping: `load_json_config(path: &str) -> Result<Vec<(String, String)>, String>`, JSON key-to-token conversion (hyphen-separated keys → equivalent CLI tokens), bool handling (`true` → flag present, `false` → no-op), unknown key silently ignored
- `src/cli/cred_parse.rs` — apply `CLR_ARGS_FILE` / `args_file` support for `isolated` and `refresh` subcommands; integrate JSON source into `apply_isolated_env_vars()` and `apply_refresh_env_vars()`
- `src/cli/help.rs` — add `--args-file <PATH>` to the help text output for `run`, `ask`, `isolated`, `refresh` subcommands; mention stdin JSON pipe detection
- `tests/json_config_test.rs` — new test file implementing all JC-1 through JC-10 test cases from `tests/docs/feature/004_json_config.md`; also implements AF-1 through AF-6 from `tests/docs/cli/param/075_args_file.md`

## Out of Scope

- TOML, YAML, or any non-JSON config format
- JSON Schema validation of the config file contents
- Adding a separate `config_param.md` CLI doc file — the feature is already documented in `docs/feature/004_json_config.md` and the flag in `docs/cli/param/075_args_file.md`; both of these doc files already exist (created in the documentation phase before this task); this task creates no new doc files
- Changing any existing CLR_* env var semantics
- Deep integration tests requiring a real `claude` subprocess (use fake binary pattern)
- GUI or TUI for editing config files

## Work Procedure

1. Read `src/cli/parse.rs` to understand `CliArgs` struct layout and `parse_args()` token-scanning loop
2. Add `args_file: Option<String>` to `CliArgs`; add `--args-file <PATH>` parsing in the token loop; register `CLR_ARGS_FILE` fallback in `apply_env_vars()` in `env.rs`
3. Create `src/cli/json_config.rs`: implement `load_json_config(path: &str) -> Result<Vec<(String, String)>, String>` that reads a JSON file, returns `(key, value)` pairs; implement `json_to_tokens(pairs: &[(String, String)]) -> Vec<String>` that converts key-value pairs to CLI token equivalents (`--key` for bools when true, `--key val` for strings/numbers); unknown keys produce no tokens; `false` booleans produce no tokens
4. In `src/cli/mod.rs` (before `parse_args()` is called), implement the injection: (a) check `args_file` field is set OR `CLR_ARGS_FILE` is set; (b) load JSON config; (c) inject JSON-derived tokens into the arg list AFTER explicit CLI tokens, BEFORE `apply_env_vars()` call — so `parse_args()` processes CLI tokens first (left-to-right first-occurrence-wins), then JSON tokens fill any remaining defaults
5. Implement stdin JSON detection: early in `run_cli()`, before parse, check `!std::io::IsTerminal::is_terminal(&std::io::stdin())` and first byte; if JSON-like, read all stdin into a string and treat as JSON parameter source (same path as args_file); gate: skip if `--file` appears in raw tokens (raw scan for `--file` before full parse)
6. Extend support to `isolated` and `refresh`: in `cred_parse.rs`, read `CLR_ARGS_FILE` and apply JSON source for subcommand-specific params
7. Write failing test cases in `tests/json_config_test.rs` following the `cli_binary_test_helpers.rs` fake claude pattern; cover all JC-1..JC-10 and AF-1..AF-6 scenarios
8. Run `w3 .test level::1` in runbox; fix all failures; iterate until green
9. Update `src/cli/help.rs` to add `--args-file <PATH>` and `CLR_ARGS_FILE` descriptions to the help output for run/ask/isolated/refresh
10. Run `w3 .test level::3` (tests + doc tests + clippy); zero warnings/failures required

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `--args-file fast.json --dry-run "task"` with JSON `{"max-sessions":0}` | args_file flag + JSON loading | dry-run preview shows `--max-sessions 0`; exit 0 |
| `--args-file fast.json --max-sessions 5 --dry-run "task"` | CLI vs JSON precedence | CLI value 5 wins over JSON value 0; exit 0 |
| `CLR_MAX_SESSIONS=5 --args-file fast.json --dry-run` with JSON `{"max-sessions":0}` | JSON vs CLR_* precedence | JSON value 0 wins over CLR_MAX_SESSIONS=5 |
| `CLR_ARGS_FILE=/path/fast.json --dry-run "task"` with valid JSON | env var fallback | JSON loaded from env var path; exit 0 |
| `echo '{"max-sessions":0}' \| clr --dry-run "task"` | stdin JSON detection | JSON consumed from stdin; max-sessions 0 applied; exit 0 |
| `--args-file /tmp/nonexistent_xyz.json "task"` | missing file error | exit 1; stderr contains error; subprocess not spawned |
| `--args-file /tmp/bad_json.json "task"` with `{invalid}` content | malformed JSON error | exit 1; stderr parse error; subprocess not spawned |
| JSON `{"dry-run": true}` with no `--dry-run` on CLI | boolean true activates flag | clr runs in dry-run mode; no subprocess spawned |
| JSON `{"_unknown": 42, "max-sessions": 0}` | unknown key ignored | no error for unknown key; max-sessions 0 applied |
| `CLR_ARGS_FILE=/path.json clr isolated --dry-run` | isolated subcommand support | JSON params applied to isolated; exit 0 |
| `clr --args-file fast.json --file input.txt "task"` | `--file` present, stdin detection skipped | `--file` takes priority; stdin NOT detected as JSON |
| `grep --args-file src/cli/help.rs` | help text update | `--args-file` appears in help output; exit 0 |

## Related Documentation

- `docs/feature/004_json_config.md` — specification: JSON parameter source, precedence rules, key format, stdin detection, ACs
- `docs/cli/param/075_args_file.md` — `--args-file` flag specification and CLR_ARGS_FILE env var
- `tests/docs/feature/004_json_config.md` — test cases JC-1 through JC-10
- `tests/docs/cli/param/075_args_file.md` — test cases AF-1 through AF-6
- `docs/cli/env_param.md` — updated precedence section (4-tier design with JSON config tier)
- `docs/feature/001_runner_tool.md` — parent feature; separation of concerns

## History

- **[2026-07-01]** `CREATED` — Implement JSON config file loading (--args-file / CLR_ARGS_FILE / stdin JSON) for all executing subcommands with CLI > JSON > CLR_* > defaults precedence.

## Verification Record

**Round 1 — PASSED (all 4 dimensions). Date: 2026-07-01.**

| Dimension | Agent | Result |
|-----------|-------|--------|
| Scope Coherence | ada64d54a9fc59c8c (initial) → a7d2b414ea4761715 (re-verify) | FAIL → PASS (findings fixed before re-verify) |
| MOST Goal Quality | abee637ef1d438efe (initial) → a1df4f47ed73fbbf2 (re-verify) | PASS → PASS |
| Value / YAGNI | a314c4469882b41df (initial) → a6da694463f1649ca (re-verify) | PASS → PASS |
| Implementation Readiness | adb6e33783afa6f40 (initial) → a38b74af32b86142f (re-verify) | PASS → PASS |

Fixes applied between Round 1 and Round 2: merged duplicate `mod.rs` In Scope entries; clarified `CLR_ARGS_FILE` ownership to `env.rs` alone; disambiguated `075_args_file.md` existence status in Out of Scope; differentiated stdin detection behavior for `isolated`/`refresh` vs `run`/`ask` subcommands.

## Verification Findings

**Round 1 — FAILED (Scope Coherence dimension). Resolved before re-verify.**

Findings addressed in this revision:
1. **Duplicate mod.rs entries** (lines 27 and 29): two separate In Scope bullets both referenced `src/cli/mod.rs` with overlapping but distinct responsibilities. Merged into a single entry with two labeled sub-parts (a) and (b).
2. **CLR_ARGS_FILE ownership ambiguity**: the `parse.rs` bullet incorrectly mentioned "parse `CLR_ARGS_FILE` env var in `apply_env_vars()` in `env.rs`" — creating a three-way inconsistency between parse.rs bullet, env.rs bullet, and Work Procedure step 2. Fixed: env.rs bullet is now the sole declared owner of CLR_ARGS_FILE reading.
3. **075_args_file.md creation status ambiguous**: Out of Scope said "Adding a dedicated `config_param.md` CLI doc file (feature documented in `feature/004` and `param/075`)" without clarifying that `075_args_file.md` already exists. Fixed: Out of Scope now explicitly states both doc files exist and this task creates no new doc files.
4. **Stdin detection for isolated/refresh undefined**: the `--file` guard for stdin detection was only described for `run`/`ask`; `isolated`/`refresh` (which don't support `--file`) were unaddressed. Fixed: mod.rs In Scope entry now explicitly states that for `isolated`/`refresh` stdin detection is always active (no `--file` guard applies).
