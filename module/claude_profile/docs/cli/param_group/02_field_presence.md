# Group :: 2. Field Presence

**Parameters:** `active::`, `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::`
**Pattern:** Per-field boolean presence control
**Purpose:** Each param independently controls whether one output line appears in text output. Shared params (`sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `role::`, `billing::`, `model::`) work identically across both commands.

| Parameter | Type | Default | Commands | Controls |
|-----------|------|---------|----------|----------|
| [`active::`](../param/13_active.md) | `bool` | `1` | `.accounts` only | Active/inactive status line |
| [`account::`](../param/05_account.md) | `bool` | `1` | `.credentials.status` only | Active account name line |
| [`sub::`](../param/06_sub.md) | `bool` | `1` | Both | Subscription type line |
| [`tier::`](../param/07_tier.md) | `bool` | `1` | Both | Rate-limit tier line |
| [`token::`](../param/08_token.md) | `bool` | `1` | `.credentials.status` only | Token status line |
| [`expires::`](../param/09_expires.md) | `bool` | `1` | Both | Token expiry duration line |
| [`email::`](../param/10_email.md) | `bool` | `1` | Both | Email address line |
| [`file::`](../param/11_file.md) | `bool` | `0` | `.credentials.status` only | Credentials file path line (opt-in) |
| [`saved::`](../param/12_saved.md) | `bool` | `0` | `.credentials.status` only | Saved account count line (opt-in) |
| [`display_name::`](../param/14_display_name.md) | `bool` | `0` | Both | Display name line (opt-in) |
| [`role::`](../param/15_role.md) | `bool` | `0` | Both | Organisation role line (opt-in) |
| [`billing::`](../param/16_billing.md) | `bool` | `0` | Both | Billing type line (opt-in) |
| [`model::`](../param/17_model.md) | `bool` | `0` | Both | Active model line (opt-in) |

**Used By (2 commands):** [`.accounts`](../command/account.md#command--3-accounts), [`.credentials.status`](../command/credentials.md#command--10-credentialsstatus)

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
clp .accounts active::0 sub::0 tier::0 expires::0 email::0

# Token-only (.credentials.status)
clp .credentials.status account::0 sub::0 tier::0 expires::0 email::0
```

**Semantic Coherence Test**

> "Does parameter X independently control ONE output field?"

All 13 members pass — each controls exactly one output line. `format::` fails (controls serialisation format, not field selection) and is correctly excluded.

**Why NOT `format::`**

- **`format::`** — selects serialisation (text, JSON, or table), not field inclusion. `format::json` and `format::table` both render all fields regardless of field-presence params — the two axes are independent.

**Cross-References**

- [../parameter_interactions.md](../parameter_interactions.md) — `format::json` override rule for field-presence params
