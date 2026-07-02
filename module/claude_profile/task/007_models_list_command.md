# Models List Command — Implementation

## Execution State

- **Executor Type:** any
- **filed_by:** agent
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🎯 (Verified)
- **closes:** null
- **dir:** src/ + claude_quota/src/
- **validated_by:** null
- **validation_date:** null

## Goal

Operators need to discover available Claude model IDs before pinning one via `.model.select` (Task 008) or passing it to `imodel::`. Without `.models`, users must consult external Anthropic documentation to obtain current model IDs — a friction point that breaks offline workflows and makes model selection error-prone. This task provides the discovery layer.

Implement the `.models` command in `claude_profile` and the `fetch_models()` / `STATIC_MODELS` infrastructure in `claude_quota`, as specified in `docs/feature/068_models_list_command.md` and `docs/cli/command/008_models.md`.

Observable end-state: `clp .models offline::1` prints a table of model IDs including `claude-opus-4-8`, `claude-sonnet-5`, and `claude-haiku-4-5-20251001`; `clp .models offline::1 format::text` prints one model ID per line with no table formatting; `clp .models offline::1 format::json` outputs a valid JSON array with `id` fields; `clp .models offline::1 name::opus` returns only models with `opus` in the ID; `clp .help` lists `.models`; `w3 .test level::3` passes with zero failures in `claude_profile`.

## In Scope

- `claude_quota/src/lib.rs` — add `ModelInfo` struct (fields: `id: String`, `display_name: String`, `created_at: Option<String>`, `max_input_tokens: Option<u64>`, `max_tokens: Option<u64>`, `capabilities: Vec<String>`); add `STATIC_MODELS: &[ModelInfo]` constant derived from `contract/claude_code/docs/model/readme.md`; add `fetch_models(token: &str) -> Result<Vec<ModelInfo>, String>` performing paginated HTTP `GET https://api.anthropic.com/v1/models` with same headers as existing quota functions (`Authorization: Bearer`, `anthropic-version: 2023-06-01`, `anthropic-beta: oauth-2025-04-20`)
- `src/commands/models.rs` — new file; `.models` command handler; reads `offline::` param to select data source; applies `name::` substring filter (case-insensitive) on `id` field; renders output in `format::table` (default), `format::text`, or `format::json`; table columns: `ID | Display Name | Context | Max Out | Ext Think`; `format::text` = one ID per line; `format::json` = serialized model array
- `src/registry.rs` — register `.models` command with `offline::`, `name::`, and `format::` parameters; register `.models` in the help group
- `tests/cli/models_test.rs` — new test file; implement all 10 test cases from `tests/docs/cli/command/19_models.md` (IT-01..IT-10) and all 10 from `tests/docs/feature/068_models_list_command.md` (FT-01..FT-10); all cases use `offline::1` to avoid network dependency in CI; use isolated temp directory pattern

## Out of Scope

- Live API mode testing (requires live credentials — not in CI scope; AC/IT coverage focuses on `offline::1`)
- Adding new crate dependencies to `claude_quota` — reuse existing HTTP client infrastructure
- Model validity checking against live API at invocation time
- `.model.select` command implementation (→ Task 008)
- Stale ID fix in `map_model_shorthand()` / `resolve_model()` (→ Task 009)

## Work Procedure

1. Read `claude_quota/src/lib.rs` to understand the existing `fetch_rate_limits()` HTTP function, header pattern, and crate structure
2. Read `contract/claude_code/docs/model/readme.md` to capture the current static model catalog entries
3. Add `ModelInfo` struct to `claude_quota/src/lib.rs`; add `STATIC_MODELS` constant populated from the catalog; ensure `ModelInfo` implements `serde::Serialize` and `serde::Deserialize`
4. Add `fetch_models(token: &str) -> Result<Vec<ModelInfo>, String>` to `claude_quota/src/lib.rs`; follow same HTTP pattern as `fetch_rate_limits()`; use `limit=1000` in the first page request to avoid pagination for the current catalog size
5. Create `src/commands/models.rs`: implement the command handler; read `offline` param (`bool`, default `false`); if offline, use `STATIC_MODELS`; otherwise call `fetch_models(token)`; read `name` param (`String`, default `""`); if non-empty, filter models by `id.to_ascii_lowercase().contains(&name.to_ascii_lowercase())`; read `format` param; render in `table`, `text`, or `json`
6. Register `.models` command in `src/registry.rs` with the three params (`offline::`, `name::`, `format::`); add to help command group
7. Write failing test cases in `tests/cli/models_test.rs` covering IT-01..IT-10 and FT-01..FT-10 scenarios; use isolated temp home directory; all tests call `clp .models offline::1`
8. Run `w3 .test level::1` in `claude_profile`; fix all failures; iterate until green
9. Run `w3 .test level::3`; fix any clippy warnings; zero warnings required

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `clp .models offline::1` | Static catalog | Stdout contains `claude-opus-4-8`; exit 0 |
| `clp .models offline::1` | Static catalog | Stdout contains `claude-sonnet-5`; exit 0 |
| `clp .models offline::1` | Static catalog | Stdout contains `claude-haiku-4-5-20251001`; exit 0 |
| `clp .models offline::1 format::table` | Table format | First line contains `ID` header; exit 0 |
| `clp .models offline::1 format::text` | Text format | Each line matches `^claude-[a-z0-9.-]+$`; no `\|` chars; exit 0 |
| `clp .models offline::1 format::json` | JSON format | Stdout parses as JSON array; each element has `"id"` field; exit 0 |
| `clp .models offline::1 name::opus` | Name filter | All returned IDs contain `opus`; haiku/sonnet absent; exit 0 |
| `clp .models offline::1 name::zz_no_match` | No-match filter | Zero model IDs in output; exit 0 |
| `clp .help` | Help registration | Output contains `.models`; exit 0 |
| `clp .models offline::1 name::claude-opus` | Substring filter | All results contain `claude-opus`; haiku/sonnet absent; exit 0 |

## Related Documentation

- `docs/feature/068_models_list_command.md` — full feature specification (AC-01..AC-10)
- `docs/cli/command/008_models.md` — CLI command spec; algorithm; parameter table
- `docs/cli/param/065_offline.md` — `offline::` parameter specification
- `tests/docs/cli/command/19_models.md` — IT-01..IT-10 integration test cases
- `tests/docs/feature/068_models_list_command.md` — FT-01..FT-10 feature test cases

## History

- **[2026-07-02]** `CREATED` — Implement `.models` command with offline/live modes, name filter, and three output formats; add `fetch_models()` and `STATIC_MODELS` to `claude_quota`.

## Verification Findings

**Round 1 — FAILED (MOST Goal Quality: M dimension). Resolved before re-verify.**

Finding: Goal section explained *what* to build but provided no *why* — no user need, no consequence of absence, no relationship to dependent tasks.

Fix applied: Added motivation paragraph before the implementation statement explaining that operators need model ID discovery before pinning and that external docs create friction/break offline workflows.

## Verification Record

**Round 2 — PASSED (all 4 dimensions). Date: 2026-07-02.**

| Dimension | Result | Agent |
|-----------|--------|-------|
| Scope Coherence | PASS | a3b24e161ac991f84 (Round 1) |
| MOST Goal Quality | PASS | adf697dd842571d51 (M re-verify Round 2) |
| Value / YAGNI | PASS | a6b2601267b0f1411 (Round 1) |
| Implementation Readiness | PASS | ac8e252a77df98714 (Round 1) |
