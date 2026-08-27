# Claude Opus 4.8

### Scope

- **Purpose**: Profile for `claude-opus-4-8` — the previous default Opus, superseded by `claude-opus-5` in v2.1.219.
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
| **Status** | Active — **superseded as default Opus by `claude-opus-5` in v2.1.219**; still fast-mode eligible |

### Workspace Usage

**No longer what `ISOLATED_DEFAULT_MODEL` resolves to.** The constant is the CLI alias `"opus"`, and since v2.1.219 that alias resolves to `claude-opus-5` — see [`013_claude_opus_5.md`](013_claude_opus_5.md).

```
ISOLATED_DEFAULT_MODEL = "opus"   // CLI alias; resolves to "claude-opus-5" since v2.1.219
```

Rationale for using an alias rather than a pinned ID: isolated subprocess runs handle high-complexity user tasks (reasoning, code generation, analysis) where capability is primary and latency secondary, and the alias auto-tracks the latest Opus with no code change. This supersession is that design working — the constant did not change; what it points at did.

The `"Resolves To"` column in `012_workspace_defaults.md § Role-to-Model Assignment` must be updated whenever Anthropic promotes a new model to the `opus` alias. That column had gone stale against v2.1.219 until this revision, which is the concrete cost of the alias indirection: nothing in the source breaks, so nothing prompts the doc update.

### Still current for

- **Fast mode.** v2.1.219 kept Opus 4.8 in `/fast` while removing Opus 4.7 from it.
- **Explicit pinning.** `--model claude-opus-4-8` still selects this model; only the unpinned `opus` alias moved.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [012_workspace_defaults.md](012_workspace_defaults.md) | Role-to-model assignment and update policy |
| source | `module/claude_runner_core/src/isolated.rs` | `ISOLATED_DEFAULT_MODEL` constant |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live model capabilities |
| doc | [013_claude_opus_5.md](013_claude_opus_5.md) | Current default Opus; what the `opus` alias resolves to now |
| doc | [001_claude_fable_5.md](001_claude_fable_5.md) | Next-tier model above Opus 4.8 |
| doc | [006_claude_opus_4_7.md](006_claude_opus_4_7.md) | Previous Opus generation; removed from fast mode in v2.1.219 |
| doc | [../version/115_v2_1_219.md](../version/115_v2_1_219.md) | Release that superseded this model as the Opus default |
