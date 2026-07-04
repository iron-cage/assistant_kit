# Claude Mythos 5

### Scope

- **Purpose**: Profile for `claude-mythos-5` — Anthropic's highest-capability model, available only through Project Glasswing.
- **Responsibility**: Documents this model's API ID, context window, max output, thinking support, access restrictions, and workspace relevance.
- **In Scope**: Model ID, alias, capabilities, access model, availability status.
- **Out of Scope**: Pricing (→ Anthropic docs); cloud platform IDs (limited availability only); model training details; Project Glasswing application process (→ Anthropic).

### Profile

| Field | Value |
|-------|-------|
| **API ID** | `claude-mythos-5` |
| **Alias** | `claude-mythos-5` |
| **Tier** | Mythos (highest capability, restricted) |
| **Context Window** | 1M tokens |
| **Max Output** | 128k tokens |
| **Extended Thinking** | No |
| **Adaptive Thinking** | Yes (always on) |
| **Effort Parameter** | Supported |
| **Latency** | — |
| **Knowledge Cutoff** | — |
| **Training Cutoff** | — |
| **GA Date** | 2026-06-09 (limited availability) |
| **Status** | Active — restricted access |

### Access Model

`claude-mythos-5` is not generally available. It is offered to approved customers through [Project Glasswing](https://anthropic.com/glasswing) — a research preview program for defensive cybersecurity workflows and high-autonomy agentic use cases. Access requires invitation from Anthropic, AWS, or Google Cloud account teams. There is no self-serve sign-up.

Predecessor: `claude-mythos-preview` (same Project Glasswing program).

Uses the tokenizer introduced with Claude Opus 4.7 — the same text produces roughly 30% more tokens compared to models before Opus 4.7.

### Workspace Usage

Not assigned any workspace role. Standard workspace OAuth credentials are unlikely to grant access to this model. Callers using `IsolatedModel::Specific("claude-mythos-5")` will receive a 403 Forbidden unless the account has Project Glasswing authorization.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [012_workspace_defaults.md](012_workspace_defaults.md) | Role-to-model assignment for workspace callers |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live model listing and capabilities |
| doc | [001_claude_fable_5.md](001_claude_fable_5.md) | Sibling widely-released Fable 5 model |
