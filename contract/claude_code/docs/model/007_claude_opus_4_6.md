# Claude Opus 4.6

### Scope

- **Purpose**: Profile for `claude-opus-4-6` — legacy Opus model; was the former `ISOLATED_DEFAULT_MODEL` before upgrade to Opus 4.8.
- **Responsibility**: Documents this model's API ID, context window, max output, thinking support, availability, and historical workspace role.
- **In Scope**: Model ID, alias, capabilities, historical workspace constant usage, availability status.
- **Out of Scope**: Pricing (→ Anthropic docs); cloud platform IDs (→ Anthropic docs); model training details.

### Profile

| Field | Value |
|-------|-------|
| **API ID** | `claude-opus-4-6` |
| **Alias** | `claude-opus-4-6` |
| **Tier** | Opus (legacy) |
| **Context Window** | 1M tokens |
| **Max Output** | 128k tokens |
| **Extended Thinking** | Yes |
| **Adaptive Thinking** | Yes |
| **Effort Parameter** | Supported |
| **Latency** | Moderate |
| **Knowledge Cutoff** | May 2025 (reliable) |
| **Training Cutoff** | Aug 2025 |
| **Status** | Legacy — consider migrating to `claude-opus-4-8` |

### Historical Workspace Role

Was the `ISOLATED_DEFAULT_MODEL` constant value in `module/claude_runner_core/src/isolated.rs` until updated to `claude-opus-4-8`. The constant was pinned to 4.6 from initial implementation through 2026-07.

The update was made because newer Opus generations (4.7, 4.8) offer improved capability at the same cost tier. Callers on 4.6 should migrate to `claude-opus-4-8`.

### Workspace Usage

No longer assigned any workspace role. Callers using `IsolatedModel::Specific("claude-opus-4-6")` will continue to work until the model is retired by Anthropic.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [003_claude_opus_4_8.md](003_claude_opus_4_8.md) | Current Opus (ISOLATED_DEFAULT_MODEL) |
| doc | [006_claude_opus_4_7.md](006_claude_opus_4_7.md) | Intermediate Opus generation |
| source | `module/claude_runner_core/src/isolated.rs` | Historical ISOLATED_DEFAULT_MODEL site |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live model listing |
