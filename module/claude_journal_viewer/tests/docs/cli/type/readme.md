# Type Tests

### Scope

- **Purpose**: Test case planning for CLI type doc instances in `docs/cli/type/`.
- **Responsibility**: Index of per-type validation test case spec files covering parsing, case sensitivity, and error handling.
- **In Scope**: All 11 `docs/cli/type/` doc instances: Duration, EventType, String, Integer, Path, OutputFormat, SortField, Boolean, GroupBy, Port, RetentionSpec.
- **Out of Scope**: Single-parameter edge cases (-> `../param/`), group interaction rules (-> `../param_group/`).

Per-type validation test case indices for `claude_journal_viewer`. See [type/readme.md](../../../../docs/cli/type/readme.md) for the source doc instances.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| `01_duration.md` | TC- tests for suffix parsing and case-sensitive M/m distinction | ✅ |
| `02_event_type.md` | TC- tests for case-insensitive enum matching across 8 variants | ✅ |
| `03_string.md` | TC- tests for unconstrained text and per-parameter additional constraints | ✅ |
| `04_integer.md` | TC- tests for non-negative parsing and range boundaries | ✅ |
| `05_path.md` | TC- tests for tilde expansion and read/write existence checks | ✅ |
| `06_output_format.md` | TC- tests for all 4 format variants and case-insensitive matching | ✅ |
| `07_sort_field.md` | TC- tests for all 6 sort fields and default directions | ✅ |
| `08_boolean.md` | TC- tests for the 0/1 convention and invalid-value rejection | ✅ |
| `09_group_by.md` | TC- tests for all 7 grouping dimensions | ✅ |
| `10_port.md` | TC- tests for ephemeral port, range limits, and bind failure | ✅ |
| `11_retention_spec.md` | TC- tests for age-based and size-based retention formats | ✅ |
