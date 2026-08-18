# Type Doc Entity

### Scope

- **Purpose**: Canonical reference for the Domain Types of `claude_profile` — the domain-meaningful values, entities, and DTOs the system is built around, independent of any single feature or command.
- **Responsibility**: Each instance documents one Domain Type: its definition and identity rules, validation/rejection rules, relationships to other Domain Types, and serialization. Feature, algorithm, and CLI docs reference these instances instead of re-defining the type per usage site.
- **In Scope**: Domain-typed values with business semantics — value objects, DDD entities (types with identity and mutable state), aggregate roots, DTOs.
- **Out of Scope**: On-disk file formats (→ `schema/`); lifecycle transitions (→ `state_machine/`); decision logic over these types (→ `algorithm/`); CLI-layer parameter value types (→ `cli/type/`); generic data structures with no domain semantics.

### Overview Table

| ID | Name | domain | ddd | Purpose | Status |
|----|------|--------|-----|---------|--------|
| — | [procedure](procedure.md) | — | — | Workflow for maintaining type instances | ✅ |
| 001 | [Account](001_account.md) | account | aggregate_root | Named credential profile — the central aggregate all operations revolve around | ✅ |
| 002 | [Identity](002_identity.md) | identity | value_object | `user@host` pair naming one acting user seat on one machine | ✅ |
| 003 | [Tag](003_tag.md) | labeling | value_object | Normalized label attached to accounts for pool partitioning | 📋 planned |
| 004 | [Tag Filter](004_tag_filter.md) | labeling | value_object | Per-Identity include/exclude tag set pair gating rotation eligibility | 📋 planned |
| 005 | [Provider](005_provider.md) | provider | value_object | Inference provider id — account credential origin and machine-global selection | ✅ |
| 006 | [Backend](006_backend.md) | account | value_object | Credential/traffic mechanism enum: `anthropic` \| `redirect` | ✅ |
| 007 | [Preset](007_preset.md) | configuration | value_object | Named save-time default bundle for known foreign providers | ✅ |
| 008 | [Quota Snapshot](008_quota_snapshot.md) | quota | dto | Point-in-time rate-limit measurement per account | ✅ |
| 009 | [Token](009_token.md) | credential | entity | Per-account credential payload — OAuth token or foreign API key | ✅ |
| 010 | [Session](010_session.md) | session | entity | Rolling usage window with per-session settings overrides | ✅ |

### Deliberately Not Instances

| Candidate | Why excluded |
|-----------|--------------|
| Model | A plain string preference threaded through settings; behavior fully owned by [algorithm/001](../algorithm/001_touch_model_selection.md), [algorithm/002](../algorithm/002_session_model_override.md), and `cli/param/` docs — no independent validation semantics |
| Sort Strategy | Closed enum whose entire meaning is the selection logic in [algorithm/007](../algorithm/007_sort_strategies.md) |
| Active Marker | Persistence artifact of Identity's current-account attribute — file format owned by [schema/005](../schema/005_active_marker.md) |
| Credential Store | Storage location, not a domain value — owned by [schema/003](../schema/003_file_topology.md), [schema/004](../schema/004_storage_root.md) |
| Role | Superseded — free-form label folded into Tag ([003](003_tag.md)); see migration note there |
