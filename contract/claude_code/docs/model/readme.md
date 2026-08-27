# Model Doc Entity

### Scope

- **Purpose**: Catalog of Claude API model IDs, aliases, and capabilities for workspace callers.
- **Responsibility**: Master file for the `model` local extension doc entity — lists all instances, declares the local type, and links to cross-collection dependencies.
- **In Scope**: All Claude API model IDs used in workspace code or returnable by `GET /v1/models`; context window and max output per model; workspace selection defaults; deprecation status.
- **Out of Scope**: Pricing (→ Anthropic docs); cloud platform IDs for Bedrock/Vertex (→ Anthropic docs); model training methodology; inference API wire contract for `/v1/messages` (→ `endpoint/003_v1_messages.md`).

### Type Declaration

**Local extension type**: `model/`

**Decision criteria**: Use `model/` when documenting a Claude API model or the model catalog — the set of valid model IDs, their properties, and their workspace usage roles. Instances describe external API resources, not workspace Domain Types.

**Why not `domain_entity/`**: `domain_entity/` documents code-level Domain Types (structs/enums in `src/`) with lifecycle and governing rulebooks. Claude API models are external API resources; they have no workspace code definition.

**Why not `endpoint/`**: `endpoint/` documents HTTP wire contracts (request + response schema). `model/` documents the model resources themselves that those endpoints return.

**Required sections** per instance:
- `### Scope` — Purpose, Responsibility, In Scope, Out of Scope (4 bullets)
- At least one content H3 section appropriate to the instance topic
- `### Cross-References` — typed table with Type, File, Responsibility columns

**Optional sections**: `### Update Procedure`, `### Known Usage in Workspace Code`

**Overview Table columns**: # | Model | API ID | Tier | Context | Max Output | Ext Thinking | Adaptive | Status

**Quality checklist**:
- [ ] All model IDs verified against `GET /v1/models` response or official Anthropic docs
- [ ] Deprecated models carry retire date
- [ ] Cross-references include both the endpoint and source code constants
- [ ] Update procedure present when constants are affected
- [ ] **Catalog completeness re-checked against the installed binary** (see below)

**Completeness check — the catalog can go stale silently.** Every existing check
above validates models that *are* listed; none detects one that is *missing*.
`claude-opus-5` shipped as the default Opus in v2.1.219 and went undocumented
here until an audit compared the catalog against the binary. Run this after any
Claude Code upgrade:

```bash
cd contract/claude_code/docs/model
V=~/.local/share/claude/versions/$(claude --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
for m in $(grep -rhoE 'claude-[a-z]+-[0-9]+(-[0-9]+)?' . | sort -u); do
  printf '%-28s docs:yes  binary:%s\n' "$m" "$(grep -ac "$m" "$V")"
done
grep -rn 'default Opus model\|default Sonnet model' ../version/ | tail -5
```

The second command is the one that catches omissions — a release note promoting
a model to a tier default names a model that must have a profile here.

### Overview Table

| # | Model | API ID | Tier | Context | Max Output | Ext Thinking | Adaptive | Status |
|---|-------|--------|------|---------|------------|--------------|----------|--------|
| [001](001_claude_fable_5.md) | claude-fable-5 | `claude-fable-5` | Fable | 1M | 128k / 300k† | No | Yes | GA |
| [002](002_claude_mythos_5.md) | claude-mythos-5 | `claude-mythos-5` | Mythos | 1M | 128k | No | Yes | invite-only |
| [003](003_claude_opus_4_8.md) | claude-opus-4-8 | `claude-opus-4-8` | Opus | 1M | 128k / 300k† | No | Yes | superseded as default (v2.1.219); fast-mode eligible |
| [004](004_claude_sonnet_5.md) | claude-sonnet-5 | `claude-sonnet-5` | Sonnet | 1M | 128k / 300k† | No | Yes | **current** |
| [005](005_claude_haiku_4_5.md) | claude-haiku-4-5 | `claude-haiku-4-5-20251001` | Haiku | 200k | 64k | Yes | No | **current** |
| [006](006_claude_opus_4_7.md) | claude-opus-4-7 | `claude-opus-4-7` | Opus‡ | 1M | 128k | No | Yes | legacy |
| [007](007_claude_opus_4_6.md) | claude-opus-4-6 | `claude-opus-4-6` | Opus | 1M | 128k | Yes | Yes | legacy |
| [008](008_claude_sonnet_4_6.md) | claude-sonnet-4-6 | `claude-sonnet-4-6` | Sonnet | 1M | 128k | Yes | Yes | legacy |
| [009](009_claude_sonnet_4_5.md) | claude-sonnet-4-5 | `claude-sonnet-4-5-20250929` | Sonnet | 200k | 64k | Yes | No | legacy |
| [010](010_claude_opus_4_5.md) | claude-opus-4-5 | `claude-opus-4-5-20251101` | Opus | 200k | 64k | Yes | No | legacy |
| [011](011_claude_opus_4_1.md) | claude-opus-4-1 | `claude-opus-4-1-20250805` | Opus | 200k | 32k | Yes | No | deprecated (retire: 2026-08-05) |
| [012](012_workspace_defaults.md) | Workspace Defaults | — | — | — | — | — | — | active |
| [013](013_claude_opus_5.md) | claude-opus-5 | `claude-opus-5` | Opus | 1M | unverified | unverified | unverified | **current default Opus** (v2.1.219+) |

† 300k via Batch API with `output-300k-2026-03-24` beta
‡ Opus 4.7 introduced a new tokenizer: same text → ~30% more tokens vs pre-4.7 models; claude-fable-5 and claude-mythos-5 share this tokenizer.

Opus 5's `unverified` cells are deliberate: no first-party source cited in this collection states its max output or thinking support, and carrying a predecessor's values forward would manufacture a fact. See [`013_claude_opus_5.md`](013_claude_opus_5.md).

**Total doc instances**: 13

### Cross-Collection Dependencies

**Endpoint**:
- `endpoint/011_v1_models.md` — wire contract for live model listing via `GET /v1/models`

**Workspace source**:
- `module/claude_runner_core/src/isolated.rs` — `ISOLATED_DEFAULT_MODEL`, `REFRESH_DEFAULT_MODEL` constants
- `module/claude_quota/src/lib.rs` — `fetch_rate_limits()` probe model constant
