# Commands: Credentials

Credential metadata commands.

---

### Command: 10. `.credentials.status`

Show live credential metadata by reading `~/.claude/.credentials.json` directly. Succeeds on any authenticated machine regardless of whether account store setup exists.

-- **Parameters:** [`format::`](../param/002_format.md), [`account::`](../param/005_account.md), [`sub::`](../param/006_sub.md), [`tier::`](../param/007_tier.md), [`token::`](../param/008_token.md), [`expires::`](../param/009_expires.md), [`email::`](../param/010_email.md), [`file::`](../param/011_file.md), [`saved::`](../param/012_saved.md), [`display_name::`](../param/014_display_name.md), [`role::`](../param/015_role.md), [`billing::`](../param/016_billing.md), [`model::`](../param/017_model.md), [`uuid::`](../param/028_uuid.md), [`capabilities::`](../param/029_capabilities.md), [`org_uuid::`](../param/030_org_uuid.md), [`org_name::`](../param/031_org_name.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 2 (credential file absent or HOME unset)

**Syntax:**

```bash
clp .credentials.status
clp .credentials.status email::0
clp .credentials.status file::1 saved::1
clp .credentials.status display_name::1 role::1 billing::1 model::1
clp .credentials.status format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format |
| `account::` | `bool` | `1` | Show active account name line |
| `sub::` | `bool` | `1` | Show subscription type line |
| `tier::` | `bool` | `1` | Show rate-limit tier line |
| `token::` | `bool` | `1` | Show token status line |
| `expires::` | `bool` | `1` | Show token expiry duration line |
| `email::` | `bool` | `1` | Show email address line |
| `file::` | `bool` | `0` | Show credentials file path (opt-in) |
| `saved::` | `bool` | `0` | Show saved account count (opt-in) |
| `display_name::` | `bool` | `0` | Show display name from `~/.claude.json` (opt-in) |
| `role::` | `bool` | `0` | Show organisation role from `~/.claude.json` (opt-in) |
| `billing::` | `bool` | `0` | Show billing type from `~/.claude.json` (opt-in) |
| `model::` | `bool` | `0` | Show active model from `~/.claude/settings.json` (opt-in) |
| `uuid::` | `bool` | `0` | Show stable user ID (`taggedId`) from `~/.claude.json` (opt-in) |
| `capabilities::` | `bool` | `0` | Show product capabilities list from `~/.claude.json` (opt-in) |
| `org_uuid::` | `bool` | `0` | Show organisation UUID from active account's `{name}.json` snapshot (opt-in) |
| `org_name::` | `bool` | `0` | Show organisation display name from active account's `{name}.json` snapshot (opt-in) |
| `trace::` | `bool` | `0` | Print timestamped diagnostic lines to stderr for the credential file read and each supplementary snapshot read |

**Algorithm (3 steps):**
1. Read `~/.claude/.credentials.json`; read `_active_{hostname}_{user}` marker (best-effort)
2. `(when snapshot fields enabled)` Read `~/.claude.json`, `~/.claude/settings.json`, and `{active_name}.json` per enabled field params (best-effort; missing files → `N/A`)
3. Render enabled fields in requested `format::`

**Examples:**

```bash
clp .credentials.status
# Account: alice@acme.com
# Sub:     max
# Tier:    default_claude_max_20x
# Token:   valid
# Expires: in 7h 24m
# Email:   N/A

clp .credentials.status file::1 saved::1
# Account: alice@acme.com
# Sub:     max
# Tier:    default_claude_max_20x
# Token:   valid
# Expires: in 7h 24m
# Email:   N/A
# File:    /home/user/.claude/.credentials.json
# Saved:   2 account(s)

clp .credentials.status format::json
# {"subscription":"max","tier":"default_claude_max_20x","token":"valid","expires_in_secs":26640,"email":"alice@acme.com","account":"alice@acme.com","file":"/home/user/.claude/.credentials.json","saved":2,"display_name":"alice","role":"admin","billing":"stripe_subscription","model":"sonnet","tagged_id":"user_01abc","capabilities":["claude_max","chat"],"organization_uuid":"aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee","organization_name":"alice@example.com's Organization","organization_role":"admin","workspace_uuid":"","workspace_name":""}
```

**Notes:**
- Field-presence params only affect text output. `format::json` always includes all fields regardless of field-presence params.
- `account::` reads the per-machine active marker; shows `N/A` on machines where no account has ever been saved.
- `saved::` counts `*.credentials.json` files in the credential store; shows `0` when the credential store is absent.

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [format::](../param/002_format.md) | Output format |
| 2 | [account::](../param/005_account.md) | Show active account name |
| 3 | [sub::](../param/006_sub.md) | Show subscription type |
| 4 | [tier::](../param/007_tier.md) | Show rate-limit tier |
| 5 | [token::](../param/008_token.md) | Show token status |
| 6 | [expires::](../param/009_expires.md) | Show token expiry duration |
| 7 | [email::](../param/010_email.md) | Show email address |
| 8 | [file::](../param/011_file.md) | Show credentials file path |
| 9 | [saved::](../param/012_saved.md) | Show saved account count |
| 10 | [display_name::](../param/014_display_name.md) | Show display name |
| 11 | [role::](../param/015_role.md) | Show organisation role |
| 12 | [billing::](../param/016_billing.md) | Show billing type |
| 13 | [model::](../param/017_model.md) | Show active model |
| 14 | [uuid::](../param/028_uuid.md) | Show stable user ID |
| 15 | [capabilities::](../param/029_capabilities.md) | Show product capabilities |
| 16 | [org_uuid::](../param/030_org_uuid.md) | Show organisation UUID |
| 17 | [org_name::](../param/031_org_name.md) | Show organisation display name |
| 18 | [trace::](../param/023_trace.md) | Diagnostic trace output |

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Live Credentials Status](../../feature/012_live_credentials_status.md) | Field set and read algorithm for live credential inspection |
| 2 | [Rich Account Metadata](../../feature/014_rich_account_metadata.md) | Extended metadata fields surfaced by this command |
| 3 | [Extended Snapshot Fields](../../feature/021_extended_snapshot_fields.md) | Opt-in snapshot fields (uuid, capabilities) |
| 4 | [Org Identity Snapshot](../../feature/022_org_identity_snapshot.md) | Org identity fields (org_uuid, org_name) |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Verify live credential state during account setup |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Primary command for live credential inspection |

### Referenced Parameter Groups

| # | Group | Parameters Used |
|---|-------|-----------------|
| 1 | [Output Control](../param_group/001_output_control.md) | `format::` |
| 2 | [Field Presence](../param_group/002_field_presence.md) | All 16 params (`account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`) |
| 3 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | `trace::` |

### Referenced Formats

| # | Format | Trigger |
|---|--------|---------|
| 1 | [text](../format/001_text.md) | `format::text` (default) |
| 2 | [json](../format/002_json.md) | `format::json` |
