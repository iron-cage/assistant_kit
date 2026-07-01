# model

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

**Overview Table columns**: ID | Name | File | Status

**Quality checklist**:
- [ ] All model IDs verified against `GET /v1/models` response or official Anthropic docs
- [ ] Deprecated models carry retire date
- [ ] Cross-references include both the endpoint and source code constants
- [ ] Update procedure present when constants are affected

### Overview Table

| ID | Name | File | Status |
|----|------|------|--------|
| [001](001_claude_fable_5.md) | claude-fable-5 | [001_claude_fable_5.md](001_claude_fable_5.md) | GA |
| [002](002_claude_mythos_5.md) | claude-mythos-5 | [002_claude_mythos_5.md](002_claude_mythos_5.md) | invite-only |
| [003](003_claude_opus_4_8.md) | claude-opus-4-8 | [003_claude_opus_4_8.md](003_claude_opus_4_8.md) | current |
| [004](004_claude_sonnet_5.md) | claude-sonnet-5 | [004_claude_sonnet_5.md](004_claude_sonnet_5.md) | current |
| [005](005_claude_haiku_4_5.md) | claude-haiku-4-5 | [005_claude_haiku_4_5.md](005_claude_haiku_4_5.md) | current |
| [006](006_claude_opus_4_7.md) | claude-opus-4-7 | [006_claude_opus_4_7.md](006_claude_opus_4_7.md) | legacy |
| [007](007_claude_opus_4_6.md) | claude-opus-4-6 | [007_claude_opus_4_6.md](007_claude_opus_4_6.md) | legacy |
| [008](008_claude_sonnet_4_6.md) | claude-sonnet-4-6 | [008_claude_sonnet_4_6.md](008_claude_sonnet_4_6.md) | legacy |
| [009](009_claude_sonnet_4_5.md) | claude-sonnet-4-5 | [009_claude_sonnet_4_5.md](009_claude_sonnet_4_5.md) | legacy |
| [010](010_claude_opus_4_5.md) | claude-opus-4-5 | [010_claude_opus_4_5.md](010_claude_opus_4_5.md) | legacy |
| [011](011_claude_opus_4_1.md) | claude-opus-4-1 | [011_claude_opus_4_1.md](011_claude_opus_4_1.md) | deprecated |
| [012](012_workspace_defaults.md) | Workspace Defaults | [012_workspace_defaults.md](012_workspace_defaults.md) | active |

**Total doc instances**: 12

### Cross-Collection Dependencies

**Endpoint**:
- `endpoint/011_v1_models.md` — wire contract for live model listing via `GET /v1/models`

**Workspace source**:
- `module/claude_runner_core/src/isolated.rs` — `ISOLATED_DEFAULT_MODEL`, `REFRESH_DEFAULT_MODEL` constants
- `module/claude_quota/src/lib.rs` — `fetch_rate_limits()` probe model constant
