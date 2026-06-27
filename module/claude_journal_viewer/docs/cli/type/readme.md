# CLI Types

### Scope

- **Purpose**: Per-type constraint and validation reference for all CLI parameter types.
- **Responsibility**: Define semantic type constraints, valid values, and parsing rules.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `01_duration.md` | Human-friendly duration parsing (1h, 7d, 4w, 3M) |
| `02_event_type.md` | 8-variant event type enum |
| `03_string.md` | UTF-8 string fundamental type |
| `04_integer.md` | Non-negative integer fundamental type |
| `05_path.md` | Filesystem path semantic type |
| `06_output_format.md` | 4-variant output format enum |
| `07_sort_field.md` | 6-variant sort field enum |
| `08_boolean.md` | 0/1 boolean fundamental type |
| `09_group_by.md` | 7-variant stats grouping enum |
| `10_port.md` | TCP port semantic type |
| `11_retention_spec.md` | Age or size retention specification |

### All Types (11 total)

| # | Type | Kind | Fundamental | Key Constraint |
|---|------|------|-------------|----------------|
| 01 | [`Duration`](01_duration.md) | Semantic | String | Suffix: s/m/h/d/w/M |
| 02 | [`EventType`](02_event_type.md) | Enum | String | 8 canonical variants |
| 03 | [`String`](03_string.md) | Fundamental | String | Any UTF-8 text |
| 04 | [`Integer`](04_integer.md) | Fundamental | Integer | Non-negative |
| 05 | [`Path`](05_path.md) | Semantic | String | Valid filesystem path |
| 06 | [`OutputFormat`](06_output_format.md) | Enum | String | 4 variants |
| 07 | [`SortField`](07_sort_field.md) | Enum | String | 6 variants |
| 08 | [`Boolean`](08_boolean.md) | Fundamental | Integer | 0 or 1 |
| 09 | [`GroupBy`](09_group_by.md) | Enum | String | 7 variants |
| 10 | [`Port`](10_port.md) | Semantic | Integer | 0-65535 |
| 11 | [`RetentionSpec`](11_retention_spec.md) | Semantic | String | Age duration or byte size |

**Total:** 11 types
