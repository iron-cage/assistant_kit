# Claude Opus 4.5

### Scope

- **Purpose**: Profile for `claude-opus-4-5-20251101` (alias `claude-opus-4-5`) — legacy Opus model with 200k context.
- **Responsibility**: Documents this model's API ID, context window, max output, thinking support, and availability status.
- **In Scope**: Model ID, alias, capabilities, availability status.
- **Out of Scope**: Pricing (→ Anthropic docs); cloud platform IDs (→ Anthropic docs); model training details.

### Profile

| Field | Value |
|-------|-------|
| **API ID** | `claude-opus-4-5-20251101` |
| **Alias** | `claude-opus-4-5` |
| **Tier** | Opus (legacy) |
| **Context Window** | 200k tokens |
| **Max Output** | 64k tokens |
| **Extended Thinking** | Yes |
| **Adaptive Thinking** | No |
| **Effort Parameter** | Not supported |
| **Latency** | Moderate |
| **Knowledge Cutoff** | May 2025 (reliable) |
| **Training Cutoff** | Aug 2025 |
| **Status** | Legacy — consider migrating to `claude-opus-4-8` |

### Workspace Usage

Not assigned any workspace role. Uses a dated ID format (`20251101` suffix); the alias `claude-opus-4-5` resolves to this snapshot.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [003_claude_opus_4_8.md](003_claude_opus_4_8.md) | Current Opus |
| doc | [007_claude_opus_4_6.md](007_claude_opus_4_6.md) | Next Opus generation |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live model listing |
