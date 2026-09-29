# Claude Fable 5

### Scope

- **Purpose**: Profile for `claude-fable-5` — Anthropic's most capable widely released model as of 2026-06.
- **Responsibility**: Documents this model's API ID, context window, max output, thinking support, availability, and workspace relevance.
- **In Scope**: Model ID, alias, capabilities, availability status, workspace usage context.
- **Out of Scope**: Pricing (→ Anthropic docs); cloud platform IDs for Bedrock/Vertex (→ Anthropic docs); model training details.

### Profile

| Field | Value |
|-------|-------|
| **API ID** | `claude-fable-5` |
| **Alias** | `claude-fable-5` |
| **Tier** | Fable (highest capability, widely released) |
| **Context Window** | 1M tokens |
| **Max Output** | 128k tokens (sync); 300k tokens (Batch API with `output-300k-2026-03-24` beta) |
| **Extended Thinking** | No |
| **Adaptive Thinking** | Yes (always on) |
| **Effort Parameter** | Supported |
| **Latency** | — |
| **Knowledge Cutoff** | — |
| **Training Cutoff** | — |
| **GA Date** | 2026-06-09 |
| **Status** | Active |

### Notes

Uses the tokenizer introduced with Claude Opus 4.7 — the same text produces roughly 30% more tokens compared to models before Opus 4.7. Account for increased token counts when migrating from pre-4.7 models.

Available on the Claude API, Claude Platform on AWS, Amazon Bedrock, Google Cloud, and Microsoft Foundry.

### Workspace Usage

Not currently assigned a workspace role (`DEFAULT_MODEL` in `module/claude_runner_core/src/isolated.rs` is `claude-opus-5-5`). `claude-fable-5` is a candidate for `DEFAULT_MODEL` reassignment if Fable becomes the standard high-capability choice over Opus. See `012_workspace_defaults.md`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [012_workspace_defaults.md](012_workspace_defaults.md) | Role-to-model assignment for workspace callers |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live model listing and capabilities |
| doc | [014_claude_opus_5_5.md](014_claude_opus_5_5.md) | Current `DEFAULT_MODEL` assignment |
