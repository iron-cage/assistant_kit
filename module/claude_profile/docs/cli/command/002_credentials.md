# Commands :: Credentials

Credential metadata commands.

---

### Command :: 10. `.credentials.status`

Show live credential metadata by reading `~/.claude/.credentials.json` directly. Succeeds on any authenticated machine regardless of whether account store setup exists.

-- **Parameters:** [`format::`](../param/002_format.md), [`account::`](../param/005_account.md), [`sub::`](../param/006_sub.md), [`tier::`](../param/007_tier.md), [`token::`](../param/008_token.md), [`expires::`](../param/009_expires.md), [`email::`](../param/010_email.md), [`file::`](../param/011_file.md), [`saved::`](../param/012_saved.md), [`display_name::`](../param/014_display_name.md), [`role::`](../param/015_role.md), [`billing::`](../param/016_billing.md), [`model::`](../param/017_model.md), [`uuid::`](../param/028_uuid.md), [`capabilities::`](../param/029_capabilities.md), [`org_uuid::`](../param/030_org_uuid.md), [`org_name::`](../param/031_org_name.md)
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
| `org_uuid::` | `bool` | `0` | Show organisation UUID from active account's `{name}.roles.json` snapshot (opt-in) |
| `org_name::` | `bool` | `0` | Show organisation display name from active account's `{name}.roles.json` snapshot (opt-in) |

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
