# Doc Entities

## Master Doc Entities Table

| Entity | Purpose | Master File | Instances |
|--------|---------|-------------|----------:|
| `api/` | Public library API contracts | [api/readme.md](api/readme.md) | 2 |
| `invariant/` | Measurable constraints on layering and cross-file consistency | [invariant/readme.md](invariant/readme.md) | 2 |
| `algorithm/` | Domain algorithms this crate implements | [algorithm/readme.md](algorithm/readme.md) | 1 |
| `pattern/` | Implementation conventions applied across the crate | [pattern/readme.md](pattern/readme.md) | 1 |

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| api | 001 | Core Surface | [api/001_core_surface.md](api/001_core_surface.md) |
| api | 002 | Version Surface | [api/002_version_surface.md](api/002_version_surface.md) |
| invariant | 001 | Layer 1 Boundary | [invariant/001_layer_one_boundary.md](invariant/001_layer_one_boundary.md) |
| invariant | 002 | Alias Literal Consistency | [invariant/002_alias_literal_consistency.md](invariant/002_alias_literal_consistency.md) |
| algorithm | 002 | Config Resolution | [algorithm/002_config_resolution.md](algorithm/002_config_resolution.md) |
| pattern | 002 | Parameter Trace | [pattern/002_parameter_trace.md](pattern/002_parameter_trace.md) |

`algorithm/` and `pattern/` each start at ID 002 rather than 001. This is deliberate: both
entities exist at two layers, and each instance keeps the same ID in both crates so a reference
to "algorithm 002" is unambiguous across the split. IDs 001 (`001_settings_type_inference.md`,
`001_version_lock.md`) describe Layer 2 concerns only and have no counterpart here — see
[readme.md](readme.md) § Relationship to `claude_version`'s docs.
