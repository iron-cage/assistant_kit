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

**No longer a workspace default.** The former `ISOLATED_DEFAULT_MODEL` held the CLI alias `"opus"`, which resolved here until v2.1.219 moved it to `claude-opus-5` ([`013_claude_opus_5.md`](013_claude_opus_5.md)). That supersession was the alias design working — the constant didn't change; what it pointed at did. The constant has since been replaced by a pinned ID:

```
DEFAULT_MODEL = "claude-opus-5-5"   // pinned full ID — not an alias
```

See [`014_claude_opus_5_5.md`](014_claude_opus_5_5.md) for why the alias was dropped.

The `"Resolves To"` column in `012_workspace_defaults.md § Role-to-Model Assignment` must be updated whenever Anthropic promotes a new model to the `opus` alias. That column had gone stale against v2.1.219 until this revision, which is the concrete cost of the alias indirection: nothing in the source breaks, so nothing prompts the doc update.

### Still current for

- **Fast mode.** v2.1.219 kept Opus 4.8 in `/fast` while removing Opus 4.7 from it.
- **Explicit pinning.** `--model claude-opus-4-8` still selects this model; only the unpinned `opus` alias moved.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [012_workspace_defaults.md](012_workspace_defaults.md) | Role-to-model assignment and update policy |
| source | `module/claude_runner_core/src/isolated.rs` | Historical `ISOLATED_DEFAULT_MODEL` site (now `DEFAULT_MODEL`) |
| endpoint | [../endpoint/011_v1_models.md](../endpoint/011_v1_models.md) | GET /v1/models — live model capabilities |
| doc | [013_claude_opus_5.md](013_claude_opus_5.md) | Current default Opus; what the `opus` alias resolves to now |
| doc | [001_claude_fable_5.md](001_claude_fable_5.md) | Next-tier model above Opus 4.8 |
| doc | [006_claude_opus_4_7.md](006_claude_opus_4_7.md) | Previous Opus generation; removed from fast mode in v2.1.219 |
| doc | [../version/115_v2_1_219.md](../version/115_v2_1_219.md) | Release that superseded this model as the Opus default |
