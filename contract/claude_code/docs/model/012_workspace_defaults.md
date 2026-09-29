# Workspace Model Defaults

### Scope

- **Purpose**: Authoritative assignment of Claude API model IDs to workspace caller roles.
- **Responsibility**: Documents which model each workspace use case targets, the selection rationale, and the update policy.
- **In Scope**: Role-to-model assignment table; rationale per role; update policy; update sequence.
- **Out of Scope**: Model capability details (→ individual model profile files `001_claude_fable_5.md` – `011_claude_opus_4_1.md`, `013_claude_opus_5.md`, `014_claude_opus_5_5.md`); API wire contract (→ `../endpoint/011_v1_models.md`); full model catalog overview (→ `readme.md § Overview Table`).

### Role-to-Model Assignment

| Role | Crate | Constant / Call Site | Value | Resolves To |
|------|-------|----------------------|-------|-------------|
| Default model (`clr isolated`; `clr run`/`ask`/`topic` level-5 default) | `claude_runner_core` | `DEFAULT_MODEL` | `"claude-opus-5-5"` (pinned full ID, not an alias) | `claude-opus-5-5` (fixed — no runtime resolution) |
| OAuth token refresh ping | `claude_runner_core` | `REFRESH_DEFAULT_MODEL` | `"claude-sonnet-5"` (pinned full ID, not an alias) | `claude-sonnet-5` (fixed — no runtime resolution) |
| Rate-limit header probe | `claude_quota` | body of `fetch_rate_limits()` | `"claude-haiku-4-5-20251001"` (full API ID) | — (sent directly to API) |

### Rationale

**Default model** (`DEFAULT_MODEL`): Isolated runs handle high-complexity user tasks — reasoning, code generation, analysis — where capability is primary, and `clr run`/`ask`/`topic` fall back to the same constant when no CLI `--model`, `--args-file`, `CLR_MODEL` or config-file `model` sets one. It replaced `ISOLATED_DEFAULT_MODEL`, which held the `"opus"` CLI alias so the subprocess tracked the latest Opus without a code change. The alias names a different model per release, per provider (the v2.1.283 catalog maps it to `claude-opus-4-6` on Foundry and `claude-opus-4-7` on a gateway) and per `ANTHROPIC_DEFAULT_OPUS_MODEL`/settings override, so one `clr` build could run different models on different machines. The explicit ID runs the same model everywhere, and changes only by source edit.

**OAuth token refresh ping** (`REFRESH_DEFAULT_MODEL`): Refresh invocations send a trivial `"."` prompt to force an OAuth token exchange. The constant is pinned to the full ID `"claude-sonnet-5"`, not an alias — it does not track future Sonnet releases automatically and must be updated by hand (see Update Policy below). Sonnet is fast and quota-efficient; Opus would waste allowance on a no-op request.

**Rate-limit probe** (body model in `fetch_rate_limits()`): Sends `max_tokens: 1` directly to the Anthropic API — not via the `claude` CLI. CLI aliases (`haiku`) are not valid API model IDs; the full dated ID `"claude-haiku-4-5-20251001"` must be used. Output is discarded. Haiku is the cheapest valid model for this purpose. Updated only if Haiku is retired.

**Alias vs full ID rule**: every workspace default is a full ID. `DEFAULT_MODEL` and `REFRESH_DEFAULT_MODEL` are both passed to the `claude` binary (via `IsolatedModel::Default` and `IsolatedModel::Specific`), which would accept an alias, but both pin explicit IDs: being CLI-bound doesn't by itself imply alias use, and a pin keeps what runs identical across releases, providers and settings. The rate-limit probe uses a full API ID because it is sent as the `"model"` field in a JSON API request body, where CLI aliases are not accepted at all.

### Update Policy

Update model defaults when:

- The assigned model ID is deprecated and approaching retire date (check individual model profile for retire date, e.g., `011_claude_opus_4_1.md`).
- A new model generation replaces the current default tier.
- A significantly more capable model in the same tier is released and the old one becomes clearly legacy.

**Update sequence**: create new model profile file in `model/` → update `readme.md` Overview Table → this file (role assignment) → source constants (see below).

**Pinned IDs trade silent drift for silent staleness.** While `ISOLATED_DEFAULT_MODEL` held the alias `"opus"`, an Anthropic-side promotion changed what it resolved to with no source edit, no compile error and no test failure — at v2.1.219 it moved from `claude-opus-4-8` to `claude-opus-5` while this table still named 4.8. Every workspace default is now a pinned ID, so none can drift silently. The cost is the opposite risk: nothing forces a source edit when a newer model ships, so each constant keeps targeting its pinned model until someone updates it by hand.

The only mechanism that notices a newer model worth adopting is a deliberate re-check. Two ways, both cheap:

```bash
# 1. Ask the binary what the "opus" alias resolves to right now — a newer ID than
#    DEFAULT_MODEL means a newer Opus has shipped (same check with --model sonnet):
claude --model opus -p 'reply with only your model id' </dev/null

# 2. Scan the installed binary's embedded changelog for tier-default promotions
#    (the version/ docs stop at v2.1.220, so they miss later ones):
V=~/.local/share/claude/versions/$(claude --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
grep -ao 'Added Claude [A-Za-z]* [0-9.]* (`[a-z0-9-]*`), now the default [A-Za-z]* model' "$V" | sort -u
```

Re-run both whenever a new Claude Code version is installed. Either one naming a model newer than the constants above is the trigger for the update sequence — no alias will pick it up on its own. The rate-limit probe is exempt: it is updated only if Haiku is retired.

### Source Constant Locations

| Constant | File | Line context |
|----------|------|--------------|
| `DEFAULT_MODEL` | `module/claude_runner_core/src/isolated.rs` | `pub const DEFAULT_MODEL : &str = "...";` |
| `REFRESH_DEFAULT_MODEL` | `module/claude_runner_core/src/isolated.rs` | `pub const REFRESH_DEFAULT_MODEL : &str = "...";` |
| probe model | `module/claude_quota/src/lib.rs` | request body JSON string in `fetch_rate_limits()` |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index and full catalog overview |
| doc | [014_claude_opus_5_5.md](014_claude_opus_5_5.md) | Current `DEFAULT_MODEL` profile |
| doc | [013_claude_opus_5.md](013_claude_opus_5.md) | What the former `ISOLATED_DEFAULT_MODEL` alias resolved to (v2.1.219–v2.1.279) |
| doc | [003_claude_opus_4_8.md](003_claude_opus_4_8.md) | What that alias resolved to before v2.1.219 |
| doc | [004_claude_sonnet_5.md](004_claude_sonnet_5.md) | Current REFRESH_DEFAULT_MODEL profile |
| doc | [005_claude_haiku_4_5.md](005_claude_haiku_4_5.md) | Current rate-limit probe model profile |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live catalog for update verification |
| source | `module/claude_runner_core/src/isolated.rs` | DEFAULT_MODEL, REFRESH_DEFAULT_MODEL |
| source | `module/claude_quota/src/lib.rs` | fetch_rate_limits() probe model |
