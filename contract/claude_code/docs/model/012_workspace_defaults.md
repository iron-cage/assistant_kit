# Workspace Model Defaults

### Scope

- **Purpose**: Authoritative assignment of Claude API model IDs to workspace caller roles.
- **Responsibility**: Documents which model each workspace use case targets, the selection rationale, and the update policy.
- **In Scope**: Role-to-model assignment table; rationale per role; update policy; update sequence.
- **Out of Scope**: Model capability details (→ individual model profile files `001_claude_fable_5.md` – `011_claude_opus_4_1.md`, `013_claude_opus_5.md`); API wire contract (→ `../endpoint/011_v1_models.md`); full model catalog overview (→ `readme.md § Overview Table`).

### Role-to-Model Assignment

| Role | Crate | Constant / Call Site | Value | Resolves To |
|------|-------|----------------------|-------|-------------|
| Isolated task execution | `claude_runner_core` | `ISOLATED_DEFAULT_MODEL` | `"opus"` (CLI alias) | `claude-opus-5` (current, since v2.1.219 — was `claude-opus-4-8`) |
| OAuth token refresh ping | `claude_runner_core` | `REFRESH_DEFAULT_MODEL` | `"sonnet"` (CLI alias) | `claude-sonnet-5` (current) |
| Rate-limit header probe | `claude_quota` | body of `fetch_rate_limits()` | `"claude-haiku-4-5-20251001"` (full API ID) | — (sent directly to API) |

### Rationale

**Isolated task execution** (`ISOLATED_DEFAULT_MODEL`): Uses the `"opus"` CLI alias so the subprocess always resolves to the latest available Opus without requiring a code change. Isolated runs handle high-complexity user tasks — reasoning, code generation, analysis — where capability is primary. The alias is a CLI feature: the `claude` binary resolves it to the current Opus model ID before making API calls.

**OAuth token refresh ping** (`REFRESH_DEFAULT_MODEL`): Refresh invocations send a trivial `"."` prompt to force an OAuth token exchange. The `"sonnet"` alias tracks the latest Sonnet automatically. Sonnet is fast and quota-efficient; Opus would waste allowance on a no-op request.

**Rate-limit probe** (body model in `fetch_rate_limits()`): Sends `max_tokens: 1` directly to the Anthropic API — not via the `claude` CLI. CLI aliases (`haiku`) are not valid API model IDs; the full dated ID `"claude-haiku-4-5-20251001"` must be used. Output is discarded. Haiku is the cheapest valid model for this purpose. Updated only if Haiku is retired.

**Alias vs full ID rule**: `ISOLATED_DEFAULT_MODEL` and `REFRESH_DEFAULT_MODEL` use CLI aliases because they are passed as `--model <value>` to the `claude` binary subprocess. The rate-limit probe uses a full API ID because it is sent as the `"model"` field in a JSON API request body, where CLI aliases are not accepted.

### Update Policy

Update model defaults when:

- The assigned model ID is deprecated and approaching retire date (check individual model profile for retire date, e.g., `011_claude_opus_4_1.md`).
- A new model generation replaces the current default tier.
- A significantly more capable model in the same tier is released and the old one becomes clearly legacy.

**Update sequence**: create new model profile file in `model/` → update `readme.md` Overview Table → this file (role assignment) → source constants (see below).

**Aliases decouple the source from the docs — and that is the failure mode.** Because `ISOLATED_DEFAULT_MODEL` and `REFRESH_DEFAULT_MODEL` hold aliases, an Anthropic-side promotion silently changes what they resolve to with no source edit, no compile error, and no test failure. Nothing in the workspace signals that the `Resolves To` column above has gone stale. That is exactly what happened between v2.1.219 (2026-07-24) and this revision: the table still named `claude-opus-4-8` after the `opus` alias had already moved to `claude-opus-5`.

The only mechanism that catches this is a deliberate re-check. Two ways, both cheap:

```bash
# 1. Ask the binary what the alias resolves to right now:
claude --model opus -p 'reply with only your model id' </dev/null

# 2. Scan the changelog for alias-tier promotions since the date on this file:
grep -rln 'default Opus model\|default Sonnet model' contract/claude_code/docs/version/
```

Re-run both whenever a new Claude Code version is installed. The rate-limit probe is exempt — it pins a full dated ID (`claude-haiku-4-5-20251001`) and therefore cannot drift.

### Source Constant Locations

| Constant | File | Line context |
|----------|------|--------------|
| `ISOLATED_DEFAULT_MODEL` | `module/claude_runner_core/src/isolated.rs` | `pub const ISOLATED_DEFAULT_MODEL : &str = "...";` |
| `REFRESH_DEFAULT_MODEL` | `module/claude_runner_core/src/isolated.rs` | `pub const REFRESH_DEFAULT_MODEL : &str = "...";` |
| probe model | `module/claude_quota/src/lib.rs` | request body JSON string in `fetch_rate_limits()` |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index and full catalog overview |
| doc | [013_claude_opus_5.md](013_claude_opus_5.md) | Current ISOLATED_DEFAULT_MODEL profile |
| doc | [003_claude_opus_4_8.md](003_claude_opus_4_8.md) | Previous ISOLATED_DEFAULT_MODEL profile (pre-v2.1.219) |
| doc | [004_claude_sonnet_5.md](004_claude_sonnet_5.md) | Current REFRESH_DEFAULT_MODEL profile |
| doc | [005_claude_haiku_4_5.md](005_claude_haiku_4_5.md) | Current rate-limit probe model profile |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live catalog for update verification |
| source | `module/claude_runner_core/src/isolated.rs` | ISOLATED_DEFAULT_MODEL, REFRESH_DEFAULT_MODEL |
| source | `module/claude_quota/src/lib.rs` | fetch_rate_limits() probe model |
