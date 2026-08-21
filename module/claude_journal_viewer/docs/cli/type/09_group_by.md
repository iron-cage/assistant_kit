# CLI Type: GroupBy

Enumeration of grouping dimensions for `.stats` aggregation.

- **Kind:** Enum
- **Fundamental:** String
- **Key Constraint:** One of 4 implemented variants (4 more planned)

### Variants

| Variant | Status | Groups Events By | Columns / Ordering |
|---------|--------|------------------|--------------------|
| `day` | Implemented | Calendar date (YYYY-MM-DD) | Count, Cost — ordered by date |
| `model` | Implemented | Claude model name | Count, Cost — ordered by name |
| `dir` | Implemented (task 543) | Working directory (`dir` field); field-less events under `(no dir)` | Count, Cost — ranked by descending count |
| `agent` | Implemented (task 543) | Agent identity (`agent_id` field, `{user}@{host}{abs_dir}/`); field-less events under `(no agent)` | Count, Cost — ranked by descending count |
| `hour` | Planned | Hour of day (00-23) | — |
| `command` | Planned | CLR command (run/ask/isolated/...) | — |
| `error` | Planned | Error class (RateLimit/Auth/...) | — |
| `creds` | Planned | Credential file name | — |

### Validation

- Exact lowercase matching (`by::MODEL` is invalid)
- Invalid variant causes exit 1 listing the implemented values: `day, model, dir, agent`

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 13 | [`by`](../param/13_by.md) |
