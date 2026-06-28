# Type Validation Tests

Type constraint tests for all 13 semantic types in `docs/cli/type/`.
Mirror of [type/](../../../../docs/cli/type/readme.md).

### Scope

- **Purpose**: Verify parsing, constraint enforcement, and error messages for each semantic type.
- **Responsibility**: TC-N validation test plans per type.
- **In Scope**: All 13 types, valid inputs, boundary values, invalid inputs.
- **Out of Scope**: Parameter edge cases (→ `param/`), command integration (→ `command/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `01_entry_count.md` | TC tests for `EntryCount` — non-negative integer |
| `02_entry_type.md` | TC tests for `EntryType` — author filter enum |
| `03_export_format.md` | TC tests for `ExportFormat` — serialization format enum |
| `04_path_substring.md` | TC tests for `PathSubstring` — case-insensitive path filter |
| `05_project_id.md` | TC tests for `ProjectId` — multi-format project identifier |
| `06_project_type.md` | TC tests for `ProjectType` — project naming scheme enum |
| `07_scope_value.md` | TC tests for `ScopeValue` — discovery boundary enum |
| `08_session_filter.md` | TC tests for `SessionFilter` — case-insensitive session substring |
| `09_session_id.md` | TC tests for `SessionId` — exact session identifier |
| `10_storage_path.md` | TC tests for `StoragePath` — filesystem path |
| `11_target_type.md` | TC tests for `TargetType` — count target enum |
| `12_topic_name.md` | TC tests for `TopicName` — session topic identifier |
| `13_strategy_type.md` | TC tests for `StrategyType` — resume strategy enum |

### Test ID Convention

| Prefix | Category | Used In |
|--------|----------|---------|
| `TC-N` | Type constraint | Type validation tests (`type/`) |

### Aggregate Counts

| File | Tests |
|------|-------|
| `01_entry_count.md` | 5 |
| `02_entry_type.md` | 5 |
| `03_export_format.md` | 5 |
| `04_path_substring.md` | 4 |
| `05_project_id.md` | 6 |
| `06_project_type.md` | 5 |
| `07_scope_value.md` | 6 |
| `08_session_filter.md` | 4 |
| `09_session_id.md` | 5 |
| `10_storage_path.md` | 4 |
| `11_target_type.md` | 5 |
| `12_topic_name.md` | 5 |
| `13_strategy_type.md` | 5 |
| **Total** | **64** |

### Related Documentation

- [type/](../../../../docs/cli/type/readme.md) — Source type specifications
- [param/](../param/) — Edge case tests that use these types
- [command/](../command/) — Integration tests (use types indirectly)
