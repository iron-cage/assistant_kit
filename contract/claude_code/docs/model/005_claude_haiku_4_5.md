# Claude Haiku 4.5

### Scope

- **Purpose**: Profile for `claude-haiku-4-5-20251001` (alias `claude-haiku-4-5`) — the fastest model with near-frontier intelligence, used as the workspace rate-limit probe model.
- **Responsibility**: Documents this model's API ID, context window, max output, thinking support, availability, and workspace role.
- **In Scope**: Model ID, alias, capabilities, workspace usage, availability status.
- **Out of Scope**: Pricing (→ Anthropic docs); cloud platform IDs for Bedrock/Vertex (→ Anthropic docs); model training details.

### Profile

| Field | Value |
|-------|-------|
| **API ID** | `claude-haiku-4-5-20251001` |
| **Alias** | `claude-haiku-4-5` |
| **Tier** | Haiku (fastest) |
| **Context Window** | 200k tokens |
| **Max Output** | 64k tokens |
| **Extended Thinking** | Yes |
| **Adaptive Thinking** | No |
| **Effort Parameter** | Not supported |
| **Latency** | Fastest |
| **Knowledge Cutoff** | Feb 2025 (reliable) |
| **Training Cutoff** | Jul 2025 |
| **Status** | Active — current Haiku |

### Workspace Usage

**Rate-limit probe model** — used in the `max_tokens: 1` request body of `module/claude_quota::fetch_rate_limits()`.

Rationale: `fetch_rate_limits()` sends a minimal `POST /v1/messages` request purely to read Anthropic rate-limit response headers. The model output is discarded. Haiku is the cheapest valid model for this purpose, minimizing quota consumption per probe.

Update only if Haiku 4.5 is retired. See `012_workspace_defaults.md`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [012_workspace_defaults.md](012_workspace_defaults.md) | Role-to-model assignment and update policy |
| source | `module/claude_quota/src/lib.rs` | Rate-limit probe model in `fetch_rate_limits()` request body |
| endpoint | [../endpoint/003_v1_messages.md](../endpoint/003_v1_messages.md) | POST /v1/messages — endpoint used for rate-limit probe |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live model capabilities |
