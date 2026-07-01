# Claude Opus 4.8

### Scope

- **Purpose**: Profile for `claude-opus-4-8` — Anthropic's most capable Opus-tier model, used as the workspace isolated-execution default.
- **Responsibility**: Documents this model's API ID, context window, max output, thinking support, availability, and workspace role.
- **In Scope**: Model ID, alias, capabilities, workspace constant assignment, availability status.
- **Out of Scope**: Pricing (→ Anthropic docs); cloud platform IDs for Bedrock/Vertex (→ Anthropic docs); model training details.

### Profile

| Field | Value |
|-------|-------|
| **API ID** | `claude-opus-4-8` |
| **Alias** | `claude-opus-4-8` |
| **Tier** | Opus (high-capability general) |
| **Context Window** | 1M tokens |
| **Max Output** | 128k tokens (sync); 300k tokens (Batch API with `output-300k-2026-03-24` beta) |
| **Extended Thinking** | No |
| **Adaptive Thinking** | Yes |
| **Effort Parameter** | Supported; defaults to `high` on all surfaces |
| **Latency** | Moderate |
| **Knowledge Cutoff** | Jan 2026 (reliable) |
| **Training Cutoff** | Jan 2026 |
| **Status** | Active — current Opus |

### Workspace Usage

**`ISOLATED_DEFAULT_MODEL`** — the `"opus"` CLI alias resolves to this model at runtime in `module/claude_runner_core/src/isolated.rs`.

Rationale: isolated subprocess runs handle high-complexity user tasks (reasoning, code generation, analysis). Opus 4.8 is the current highest-capability general-availability Opus. Latency is secondary; quality is primary.

```
ISOLATED_DEFAULT_MODEL = "opus"   // CLI alias; resolves to "claude-opus-4-8" currently
```

The `"opus"` alias auto-tracks the latest Opus — no code change needed when a new Opus is released. The `"Resolves To"` column in `012_workspace_defaults.md § Role-to-Model Assignment` is updated whenever Anthropic promotes a new model to the `opus` alias. See `012_workspace_defaults.md` for update policy.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [012_workspace_defaults.md](012_workspace_defaults.md) | Role-to-model assignment and update policy |
| source | `module/claude_runner_core/src/isolated.rs` | `ISOLATED_DEFAULT_MODEL` constant |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live model capabilities |
| doc | [001_claude_fable_5.md](001_claude_fable_5.md) | Next-tier model above Opus 4.8 |
| doc | [006_claude_opus_4_7.md](006_claude_opus_4_7.md) | Previous Opus generation |
