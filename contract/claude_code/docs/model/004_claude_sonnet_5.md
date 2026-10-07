# Claude Sonnet 5

### Scope

- **Purpose**: Profile for `claude-sonnet-5` — the best combination of speed and intelligence, what the `"sonnet"` CLI alias resolves to on first-party auth, and the workspace's built-in `DEFAULT_MODEL` and token-refresh default.
- **Responsibility**: Documents this model's API ID, context window, max output, thinking support, availability, and workspace role.
- **In Scope**: Model ID, alias, capabilities, workspace constant assignment, availability status.
- **Out of Scope**: Pricing (→ Anthropic docs); cloud platform IDs for Bedrock/Vertex (→ Anthropic docs); model training details.

### Profile

| Field | Value |
|-------|-------|
| **API ID** | `claude-sonnet-5` |
| **Alias** | `sonnet` resolves here on first-party auth. The v2.1.283 catalog maps the same alias to `claude-sonnet-4-5` on Bedrock, Vertex, Foundry and Mantle, and to `claude-sonnet-4-6` on anthropic_aws and a gateway |
| **Tier** | Sonnet (speed + intelligence balance) |
| **Context Window** | 1M tokens |
| **Max Output** | 128k tokens (sync) — catalog `max_output_tokens` upper bound `128000`, default `64000`; 300k tokens (Batch API with `output-300k-2026-03-24` beta) |
| **Extended Thinking** | No |
| **Adaptive Thinking** | Yes |
| **Effort Parameter** | Supported — catalog capabilities `effort`, `xhigh_effort`, `max_effort`; defaults to `high` on the Claude API and Claude Code (catalog `default_effort`) |
| **Latency** | Fast |
| **Knowledge Cutoff** | Jan 2026 (reliable) |
| **Training Cutoff** | Jan 2026 |
| **Status** | Active — current Sonnet |

The catalog values above (alias mapping, max-output bounds, effort capabilities) come from the model catalog baked into the installed v2.1.283 binary; see § Verification to re-read them.

### Workspace Usage

**`DEFAULT_MODEL`** (`module/claude_runner_core/src/isolated.rs`) is the explicit ID `"claude-sonnet-5"`, not the `sonnet` alias. It is the target of `IsolatedModel::Default` (`clr isolated`) and the level-5 built-in default of `clr run`/`ask`/`topic`. The alias would name a different model per release, per provider (Profile above), and per `ANTHROPIC_DEFAULT_SONNET_MODEL` override, so the constant pins the ID and changes only by source edit. Its predecessor `ISOLATED_DEFAULT_MODEL` was the `"opus"` alias.

Rationale: these are real user tasks, and Sonnet trades Opus's peak capability for speed and lower per-token cost — `--model claude-opus-5-5` still runs Opus. It accepts every default `clr` layers on top of the model: `--effort max` (`max_effort`), `CLAUDE_CODE_MAX_OUTPUT_TOKENS=128000` (the catalog upper bound) and the 400k auto-compact window (below the 1M context).

**`REFRESH_DEFAULT_MODEL`** — pinned directly to this model's full ID in `module/claude_runner_core/src/isolated.rs`, not a CLI alias. It is passed via `IsolatedModel::Specific`, the same pinned-ID path used for any exact model selection. It holds the same value as `DEFAULT_MODEL` today but stays a separate constant: each follows its own target.

Rationale: credential refresh invocations send a trivial `"."` prompt to force an OAuth token exchange. Sonnet 5 is fast and quota-efficient. Using Opus would waste allowance on a no-op request where output is discarded.

```
DEFAULT_MODEL         = "claude-sonnet-5"   // pinned full ID — not an alias
REFRESH_DEFAULT_MODEL = "claude-sonnet-5"   // pinned full ID — not an alias
```

Neither constant auto-tracks future Sonnet releases: the source's own rustdoc for each states "Update this constant when the target model changes." `REFRESH_DEFAULT_MODEL` briefly held the alias `"sonnet"` for about 2.5 hours during same-day development on 2026-07-02 (commit `804bc4d4`, reverted in `5cd1d9bb`) — this file described that transient state and was never reconciled against the revert until now. See `012_workspace_defaults.md` for update policy.

### Verification

```bash
V=~/.local/share/claude/versions/$(claude --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')

# Catalog entry (provenance for the Profile table's catalog values):
grep -ao '{id:"claude-sonnet-5",family:[^]]*][^}]*' "$V" | head -1 \
  | grep -oE 'max_output_tokens:\{[^}]*\}|default_effort:"[a-z]+"|knowledge_cutoff:"[^"]*"|window:[0-9e]+|"effort"|"xhigh_effort"|"max_effort"|"adaptive_thinking"'
grep -ao '{id:"claude-totally-fake-9",family:' "$V" | wc -l   # → 0 (negative control)

# Alias resolution per provider:
grep -ao 'sonnet:{default:"[^"]*",per_provider:{[^}]*}' "$V" | head -1
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [012_workspace_defaults.md](012_workspace_defaults.md) | Role-to-model assignment and update policy |
| source | `module/claude_runner_core/src/isolated.rs` | `DEFAULT_MODEL`, `REFRESH_DEFAULT_MODEL` constants |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live model capabilities |
| doc | [008_claude_sonnet_4_6.md](008_claude_sonnet_4_6.md) | Previous Sonnet generation (was REFRESH_DEFAULT_MODEL) |
| doc | [014_claude_opus_5_5.md](014_claude_opus_5_5.md) | Current default Opus (`--model claude-opus-5-5`) |
