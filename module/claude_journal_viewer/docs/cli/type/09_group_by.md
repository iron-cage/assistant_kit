# CLI Type: GroupBy

Enumeration of grouping dimensions for `.stats` aggregation.

- **Kind:** Enum
- **Fundamental:** String
- **Key Constraint:** One of 7 variants

### Variants

| Variant | Groups Events By | Typical Columns |
|---------|------------------|-----------------|
| `day` | Calendar date (YYYY-MM-DD) | Count, OK, Fail, Cost, Tokens |
| `hour` | Hour of day (00-23) | Count, OK, Fail, Avg Duration |
| `model` | Claude model name | Count, Cost, Tokens In/Out |
| `command` | CLR command (run/ask/isolated/...) | Count, OK, Fail, Avg Duration |
| `error` | Error class (RateLimit/Auth/...) | Count, Retries, Last Seen |
| `creds` | Credential file name | Count, Cost, Duration |
| `dir` | Working directory | Count, Cost, Duration |

### Validation

- Case-insensitive matching
- Invalid variant causes exit 1 listing valid options

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 13 | [`by`](../param/13_by.md) |
