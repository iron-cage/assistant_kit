# Claude Opus 4.1

### Scope

- **Purpose**: Profile for `claude-opus-4-1-20250805` (alias `claude-opus-4-1`) — deprecated Opus model retiring 2026-08-05.
- **Responsibility**: Documents this model's API ID, context window, max output, thinking support, deprecation timeline, and migration guidance.
- **In Scope**: Model ID, alias, capabilities, deprecation date, migration target.
- **Out of Scope**: Pricing (→ Anthropic docs); cloud platform IDs (→ Anthropic docs); model training details.

### Profile

| Field | Value |
|-------|-------|
| **API ID** | `claude-opus-4-1-20250805` |
| **Alias** | `claude-opus-4-1` |
| **Tier** | Opus (deprecated) |
| **Context Window** | 200k tokens |
| **Max Output** | 32k tokens |
| **Extended Thinking** | Yes |
| **Adaptive Thinking** | No |
| **Effort Parameter** | Not supported |
| **Latency** | Moderate |
| **Knowledge Cutoff** | Jan 2025 (reliable) |
| **Training Cutoff** | Mar 2025 |
| **Deprecation Announced** | Apr 14, 2026 |
| **Retire Date** | 2026-08-05 |
| **Status** | DEPRECATED |

### Migration

Anthropic notified developers of this model's retirement on April 14, 2026. Callers must migrate to `claude-opus-4-8` (or higher) before August 5, 2026. After the retire date, requests to this model ID will fail with an error.

Any `IsolatedModel::Specific("claude-opus-4-1-20250805")` or `IsolatedModel::Specific("claude-opus-4-1")` calls must be updated. No workspace constants currently reference this model.

### Workspace Usage

Not assigned any workspace role. This model was never the `ISOLATED_DEFAULT_MODEL` — workspace constants went from `claude-opus-4-6` directly to `claude-opus-4-8`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [003_claude_opus_4_8.md](003_claude_opus_4_8.md) | Recommended migration target |
| doc | [010_claude_opus_4_5.md](010_claude_opus_4_5.md) | Next Opus generation |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live model listing |
