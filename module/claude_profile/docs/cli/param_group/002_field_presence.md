# Group :: 2. Field Presence

**Parameters:** `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`
**Pattern:** Per-field boolean presence control
**Purpose:** Each param independently controls whether one output line appears in text output. Shared params (`sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`) work identically across both commands.

| Parameter | Type | Default | Commands | Controls |
|-----------|------|---------|----------|----------|
| [`account::`](../param/005_account.md) | `bool` | `1` | `.credentials.status` only | Active account name line |
| [`sub::`](../param/006_sub.md) | `bool` | `1` | Both | Subscription type line |
| [`tier::`](../param/007_tier.md) | `bool` | `1` | Both | Rate-limit tier line |
| [`token::`](../param/008_token.md) | `bool` | `1` | `.credentials.status` only | Token status line |
| [`expires::`](../param/009_expires.md) | `bool` | `1` | Both | Token expiry duration line |
| [`email::`](../param/010_email.md) | `bool` | `1` | Both | Email address line |
| [`file::`](../param/011_file.md) | `bool` | `0` | `.credentials.status` only | Credentials file path line (opt-in) |
| [`saved::`](../param/012_saved.md) | `bool` | `0` | `.credentials.status` only | Saved account count line (opt-in) |
| [`display_name::`](../param/014_display_name.md) | `bool` | `0` | Both | Display name line (opt-in) |
| [`role::`](../param/015_role.md) | `bool` | `0` | Both | Organisation role line (opt-in) |
| [`billing::`](../param/016_billing.md) | `bool` | `0` | Both | Billing type line (opt-in) |
| [`model::`](../param/017_model.md) | `bool` | `0` | Both | Active model line (opt-in) |
| [`uuid::`](../param/028_uuid.md) | `bool` | `0` | Both | Stable user ID line (opt-in) |
| [`capabilities::`](../param/029_capabilities.md) | `bool` | `0` | Both | Product capabilities list line (opt-in) |
| [`org_uuid::`](../param/030_org_uuid.md) | `bool` | `0` | Both | Organisation UUID line (opt-in) |
| [`org_name::`](../param/031_org_name.md) | `bool` | `0` | Both | Organisation display name line (opt-in) |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md#command--3-accounts) | 12 field-presence params |
| 2 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | 16 field-presence params |

**Typical Patterns:**

```bash
# Default: all on-by-default fields
clp .accounts
clp .credentials.status

# Compact: suppress less-essential fields
clp .accounts sub::0 tier::0 email::0
clp .credentials.status email::0

# Debug .credentials.status: add file path and account count
clp .credentials.status file::1 saved::1

# Bare names only (.accounts)
clp .accounts sub::0 tier::0 expires::0 email::0

# Token-only (.credentials.status)
clp .credentials.status account::0 sub::0 tier::0 expires::0 email::0
```

**Semantic Coherence Test**

> "Does parameter X independently control ONE output field?"

All 16 members pass — each controls exactly one output line. `format::` fails (controls serialisation format, not field selection) and is correctly excluded.

**Why NOT `format::`**

- **`format::`** — selects serialisation (text, JSON, or table), not field inclusion. `format::json` and `format::table` both render all fields regardless of field-presence params — the two axes are independent.

**Cross-References**

- [../004_parameter_interactions.md](../004_parameter_interactions.md) — `format::json` override rule for field-presence params

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Selective field display during account management |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Field-filtered credential diagnostic output |
