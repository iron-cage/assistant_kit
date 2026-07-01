# User Story: 4. Scripted Pipeline Automation

**Persona:** DevOps engineer integrating `clp` into CI/CD pipelines and shell scripts
**Goal:** Extract account state as structured data for scripted decision-making without parsing fragile text
**Benefit:** Enables reliable automation — quota-gated job scheduling, CI health checks, account selection scripts
**Priority:** Medium

### Acceptance Criteria

- [ ] `format::json` on any format-capable command returns valid JSON (object or array)
- [ ] `get::FIELD` returns a single bare scalar value with no headers for shell variable capture
- [ ] Exit codes are deterministic and documented — scripts can branch on them without parsing output
- [ ] `only_next::1 get::account` returns the recommended account as a bare string

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Primary: quota data via `format::json` / `get::` |
| 2 | [`.accounts`](../command/001_account.md#command--3-accounts) | Enumerate all accounts as JSON |
| 3 | [`.token.status`](../command/005_token.md#command--7-tokenstatus) | Token validity check in automated health checks |
| 4 | [`.account.rotate`](../command/001_account.md#command--13-accountrotate) | Automated account selection in pipelines |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`format::`](../param/002_format.md) | JSON/value/tsv/plain for pipeline consumption |
| 2 | [`get::`](../param/045_get.md) | Bare scalar extraction for shell variable capture |
| 3 | [`only_next::`](../param/040_only_next.md) | Return only the recommended-next row |
| 4 | [`only_active::`](../param/039_only_active.md) | Return only the active account row |
| 5 | [`no_color::`](../param/047_no_color.md) | Strip ANSI/emoji for log-safe output |
| 6 | [`dry::`](../param/004_dry.md) | Validate scripts without side effects |
| 7 | [`count::`](../param/037_count.md) | Limit rows returned |
| 8 | [`offset::`](../param/038_offset.md) | Skip first N rows for pagination |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Output Control](../param_group/001_output_control.md) | `format::`, `get::` — structured output and extraction |
| 2 | [Display Control](../param_group/005_display_control.md) | Row filtering for targeted automation queries |

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [`json`](../format/002_json.md) | Primary structured output for pipeline use |
| 2 | [`text`](../format/001_text.md) | Default fallback for human-readable scripts |
