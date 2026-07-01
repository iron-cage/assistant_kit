# Claude Opus 4.7

### Scope

- **Purpose**: Profile for `claude-opus-4-7` — legacy Opus model still available on the API.
- **Responsibility**: Documents this model's API ID, context window, max output, thinking support, availability, and workspace relevance.
- **In Scope**: Model ID, alias, capabilities, migration guidance, availability status.
- **Out of Scope**: Pricing (→ Anthropic docs); cloud platform IDs (→ Anthropic docs); model training details.

### Profile

| Field | Value |
|-------|-------|
| **API ID** | `claude-opus-4-7` |
| **Alias** | `claude-opus-4-7` |
| **Tier** | Opus (legacy) |
| **Context Window** | 1M tokens |
| **Max Output** | 128k tokens |
| **Extended Thinking** | No |
| **Adaptive Thinking** | Yes |
| **Effort Parameter** | Supported |
| **Latency** | Moderate |
| **Knowledge Cutoff** | Jan 2026 (reliable) |
| **Training Cutoff** | Jan 2026 |
| **Status** | Legacy — consider migrating to `claude-opus-4-8` |

### Notes

Introduced a new tokenizer — the same text produces roughly 30% more tokens compared to models before Opus 4.7. Models `claude-fable-5` and `claude-mythos-5` also use this tokenizer.

### Workspace Usage

Not assigned any workspace role. Workspace constants (`ISOLATED_DEFAULT_MODEL`) were updated from `claude-opus-4-6` to `claude-opus-4-8`, skipping Opus 4.7. Callers may pass `IsolatedModel::Specific("claude-opus-4-7")` if 4.7-specific behavior is required.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [003_claude_opus_4_8.md](003_claude_opus_4_8.md) | Current Opus (successor) |
| doc | [007_claude_opus_4_6.md](007_claude_opus_4_6.md) | Previous Opus generation |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live model listing |
