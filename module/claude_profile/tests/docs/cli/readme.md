# Testing

Test case planning for clp CLI. Each file contains a Test Case Index with coverage summary. Detailed test sections (executable specs) are added at L5.

### Scope

- **Purpose**: Document integration and edge case test plans for all clp commands and parameters.
- **Responsibility**: Index of per-command, per-parameter, and per-group test case planning files.
- **In Scope**: All 16 active clp commands (14 visible + 2 hidden/DEPRECATED) plus binary meta-flags (`--version`/`-V`); 2 REMOVED commands covered as regression specs (`.account.assign`, `.account.unclaim`); 58 active parameters — param slot numbering note: slot 2 retired, slot 56 unoccupied (tests slots 57–64 correspond to docs slots 056–063); tombstone slots: 14, 32, 53, 57, 58 (`active::`, `next::`, `for::`, `unclaim::`, `assign::` REMOVED); `current::` covered by command IT tests; and all 6 parameter groups.
- **Out of Scope**: Automated test implementations (→ `tests/` in crate), spec documentation (→ `docs/feature/`).

### Responsibility Table

| Directory | Responsibility |
|-----------|----------------|
| command/ | Per-command integration test case indices (IT-N entries) |
| param/ | Per-parameter edge case indices (EC-N entries) |
| param_group/ | Per-parameter-group interaction test indices |
| type/ | Per-type acceptance and boundary case indices (TC-N entries) |
| command_verb/ | Per-verb behavioral contract specs (BV-N entries) |
| command_noun/ | Per-noun lifecycle and schema contract specs (NC-N entries) |
| user_story/ | User acceptance scenario specs (UA-N entries) |
| format/ | Per-format behavioral contract specs (FM-N entries) |

### Coverage Summary

| Scope | Files | Min Tests |
|-------|-------|-----------|
| Commands + meta-flags | 19 | >=8 IT each (16 active + 2 REMOVED regression + 1 meta-flag) |
| Parameters | 58 | >=6 EC each |
| Parameter groups | 6 | >=4 IT each |
| Command verbs | 11 | >=3 BV each (9 active + 2 REMOVED regression) |
| Command nouns | 3 | >=3 NC each |
| User stories | 5 | 4–6 UA each |
| Output formats | 3 | >=4 FM each |

### Navigation

**Commands:**
- [`--version` / `-V`](command/00_version.md)
- [`.` (help alias)](command/01_dot.md)
- [`.help`](command/02_help.md)
- [`.accounts`](command/03_accounts.md)
- [`.account.save`](command/04_account_save.md)
- [`.account.use`](command/05_account_use.md)
- [`.account.delete`](command/06_account_delete.md)
- [`.token.status`](command/07_token_status.md)
- [`.paths`](command/08_paths.md)
- [`.usage`](command/09_usage.md)
- [`.credentials.status`](command/10_credentials_status.md)
- [`.account.limits`](command/11_account_limits.md)
- [`.account.relogin`](command/12_account_relogin.md)
- [`.account.rotate`](command/13_account_rotate.md)
- [`.account.renewal`](command/14_account_renewal.md)
- [`.account.inspect`](command/15_account_inspect.md)
- [`.account.assign` *(REMOVED)*](command/16_account_assign.md)
- [`.model`](command/17_model.md)
- [`.account.unclaim` *(REMOVED)*](command/18_account_unclaim.md)

**Parameters:**
- [`name::`](param/01_name.md)
- [`format::` / `fmt::`](param/03_format.md)
- [`threshold::`](param/04_threshold.md)
- [`dry::`](param/05_dry.md)
- [`account::`](param/06_account.md)
- [`sub::`](param/07_sub.md)
- [`tier::`](param/08_tier.md)
- [`token::`](param/09_token.md)
- [`expires::`](param/10_expires.md)
- [`email::`](param/11_email.md)
- [`file::`](param/12_file.md)
- [`saved::`](param/13_saved.md)
- [`active::` *(REMOVED)*](param/14_active.md)
- [`display_name::`](param/15_display_name.md)
- [`role::`](param/16_role.md)
- [`billing::`](param/17_billing.md)
- [`model::`](param/18_model.md)
- [`refresh::`](param/19_refresh.md)
- [`live::`](param/20_live.md)
- [`interval::`](param/21_interval.md)
- [`jitter::`](param/22_jitter.md)
- [`trace::`](param/23_trace.md)
- [`field::`](param/24_field.md)
- [`sort::`](param/25_sort.md)
- [`desc::`](param/26_desc.md)
- [`prefer::`](param/27_prefer.md)
- [`uuid::`](param/28_uuid.md)
- [`capabilities::`](param/29_capabilities.md)
- [`org_uuid::`](param/30_org_uuid.md)
- [`org_name::`](param/31_org_name.md)
- [`next::` *(REMOVED)*](param/32_next.md)
- [`cols::`](param/33_cols.md)
- [`touch::`](param/34_touch.md)
- [`imodel::`](param/35_imodel.md)
- [`effort::`](param/36_effort.md)
- [`count::`](param/37_count.md)
- [`offset::`](param/38_offset.md)
- [`only_active::`](param/39_only_active.md)
- [`only_next::`](param/40_only_next.md)
- [`min_5h::`](param/41_min_5h.md)
- [`min_7d::`](param/42_min_7d.md)
- [`only_valid::`](param/43_only_valid.md)
- [`exclude_exhausted::`](param/44_exclude_exhausted.md)
- [`get::`](param/45_get.md)
- [`abs::`](param/46_abs.md)
- [`no_color::`](param/47_no_color.md)
- [`host::`](param/48_host.md)
- [`at::`](param/49_at.md)
- [`from_now::`](param/50_from_now.md)
- [`clear::`](param/51_clear.md)
- [`role::` (metadata label)](param/52_role.md)
- [`for::` *(REMOVED)*](param/53_for.md)
- [`set_model::`](param/54_set_model.md)
- [`set::`](param/55_set.md)
- [`unclaim::` *(REMOVED)*](param/57_unclaim.md)
- [`assign::` *(REMOVED)*](param/58_assign.md)
- [`force::`](param/59_force.md)
- [`rotate::`](param/60_rotate.md)
- [`solo::`](param/61_solo.md)
- [`who::`](param/62_who.md)
- [`owner::`](param/63_owner.md)
- [`assignee::`](param/64_assignee.md)

**Types:**
- [AccountName](type/01_account_name.md)
- [OutputFormat](type/02_output_format.md)
- [WarningThreshold](type/03_warning_threshold.md)
- [AccountSelector](type/04_account_selector.md)

**Parameter Groups:**
- [Output Control](param_group/01_output_control.md)
- [Field Presence](param_group/02_field_presence.md)
- [Fetch Behavior](param_group/03_fetch_behavior.md)
- [Sort Control](param_group/04_sort_control.md)
- [Display Control](param_group/05_display_control.md)
- [Account Targeting](param_group/06_account_targeting.md)

**Command Verbs:**
- [command_verb/](command_verb/readme.md)

**Command Nouns:**
- [command_noun/](command_noun/readme.md)

**User Stories:**
- [user_story/](user_story/readme.md)

**Output Formats:**
- [format/](format/readme.md)
