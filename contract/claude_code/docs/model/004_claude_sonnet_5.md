# Claude Sonnet 5

### Scope

- **Purpose**: Profile for `claude-sonnet-5` — the best combination of speed and intelligence, used as the workspace token-refresh default.
- **Responsibility**: Documents this model's API ID, context window, max output, thinking support, availability, and workspace role.
- **In Scope**: Model ID, alias, capabilities, workspace constant assignment, availability status.
- **Out of Scope**: Pricing (→ Anthropic docs); cloud platform IDs for Bedrock/Vertex (→ Anthropic docs); model training details.

### Profile

| Field | Value |
|-------|-------|
| **API ID** | `claude-sonnet-5` |
| **Alias** | `claude-sonnet-5` |
| **Tier** | Sonnet (speed + intelligence balance) |
| **Context Window** | 1M tokens |
| **Max Output** | 128k tokens (sync); 300k tokens (Batch API with `output-300k-2026-03-24` beta) |
| **Extended Thinking** | No |
| **Adaptive Thinking** | Yes |
| **Effort Parameter** | Supported; defaults to `high` on the Claude API and Claude Code |
| **Latency** | Fast |
| **Knowledge Cutoff** | Jan 2026 (reliable) |
| **Training Cutoff** | Jan 2026 |
| **Status** | Active — current Sonnet |

### Workspace Usage

**`REFRESH_DEFAULT_MODEL`** — the `"sonnet"` CLI alias resolves to this model at runtime in `module/claude_runner_core/src/isolated.rs`.

Rationale: credential refresh invocations send a trivial `"."` prompt to force an OAuth token exchange. Sonnet 5 is fast and quota-efficient. Using Opus would waste allowance on a no-op request where output is discarded.

```
REFRESH_DEFAULT_MODEL = "sonnet"   // CLI alias; resolves to "claude-sonnet-5" currently
```

The `"sonnet"` alias auto-tracks the latest Sonnet — no code change needed when a new Sonnet is released. The `"Resolves To"` column in `012_workspace_defaults.md § Role-to-Model Assignment` is updated whenever Anthropic promotes a new model to the `sonnet` alias. See `012_workspace_defaults.md` for update policy.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [012_workspace_defaults.md](012_workspace_defaults.md) | Role-to-model assignment and update policy |
| source | `module/claude_runner_core/src/isolated.rs` | `REFRESH_DEFAULT_MODEL` constant |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live model capabilities |
| doc | [008_claude_sonnet_4_6.md](008_claude_sonnet_4_6.md) | Previous Sonnet generation (was REFRESH_DEFAULT_MODEL) |
