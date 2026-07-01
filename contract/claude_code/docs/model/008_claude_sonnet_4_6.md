# Claude Sonnet 4.6

### Scope

- **Purpose**: Profile for `claude-sonnet-4-6` — legacy Sonnet model; was the former `REFRESH_DEFAULT_MODEL` before upgrade to Sonnet 5.
- **Responsibility**: Documents this model's API ID, context window, max output, thinking support, availability, and historical workspace role.
- **In Scope**: Model ID, alias, capabilities, historical workspace constant usage, availability status.
- **Out of Scope**: Pricing (→ Anthropic docs); cloud platform IDs (→ Anthropic docs); model training details.

### Profile

| Field | Value |
|-------|-------|
| **API ID** | `claude-sonnet-4-6` |
| **Alias** | `claude-sonnet-4-6` |
| **Tier** | Sonnet (legacy) |
| **Context Window** | 1M tokens |
| **Max Output** | 128k tokens |
| **Extended Thinking** | Yes |
| **Adaptive Thinking** | Yes |
| **Effort Parameter** | Supported |
| **Latency** | Fast |
| **Knowledge Cutoff** | Aug 2025 (reliable) |
| **Training Cutoff** | Jan 2026 |
| **Status** | Legacy — consider migrating to `claude-sonnet-5` |

### Historical Workspace Role

Was the `REFRESH_DEFAULT_MODEL` constant value in `module/claude_runner_core/src/isolated.rs` until updated to `claude-sonnet-5` in 2026-07. Used for OAuth credential-refresh pings (trivial `"."` prompt).

### Workspace Usage

No longer assigned any workspace role. Callers using `IsolatedModel::Specific("claude-sonnet-4-6")` will continue to work until the model is retired.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [004_claude_sonnet_5.md](004_claude_sonnet_5.md) | Current Sonnet (REFRESH_DEFAULT_MODEL) |
| source | `module/claude_runner_core/src/isolated.rs` | Historical REFRESH_DEFAULT_MODEL site |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live model listing |
