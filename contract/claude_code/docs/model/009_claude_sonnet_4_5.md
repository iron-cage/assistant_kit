# Claude Sonnet 4.5

### Scope

- **Purpose**: Profile for `claude-sonnet-4-5-20250929` (alias `claude-sonnet-4-5`) — legacy Sonnet model with 200k context.
- **Responsibility**: Documents this model's API ID, context window, max output, thinking support, and availability status.
- **In Scope**: Model ID, alias, capabilities, availability status.
- **Out of Scope**: Pricing (→ Anthropic docs); cloud platform IDs (→ Anthropic docs); model training details.

### Profile

| Field | Value |
|-------|-------|
| **API ID** | `claude-sonnet-4-5-20250929` |
| **Alias** | `claude-sonnet-4-5` |
| **Tier** | Sonnet (legacy) |
| **Context Window** | 200k tokens |
| **Max Output** | 64k tokens |
| **Extended Thinking** | Yes |
| **Adaptive Thinking** | No |
| **Effort Parameter** | Not supported |
| **Latency** | Fast |
| **Knowledge Cutoff** | Jan 2025 (reliable) |
| **Training Cutoff** | Jul 2025 |
| **Status** | Legacy — consider migrating to `claude-sonnet-5` |

### Workspace Usage

Not assigned any workspace role. Uses a dated ID format (`20250929` suffix); the alias `claude-sonnet-4-5` resolves to this snapshot.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [004_claude_sonnet_5.md](004_claude_sonnet_5.md) | Current Sonnet |
| doc | [008_claude_sonnet_4_6.md](008_claude_sonnet_4_6.md) | Next Sonnet generation |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live model listing |
