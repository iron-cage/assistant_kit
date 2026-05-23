# Commands :: Credentials

Credential metadata commands.

---

### Command :: 10. `.credentials.status`

Show live credential metadata by reading `~/.claude/.credentials.json` directly. Succeeds on any authenticated machine regardless of whether account store setup exists.

-- **Parameters:** [`format::`](../param/02_format.md), [`account::`](../param/05_account.md), [`sub::`](../param/06_sub.md), [`tier::`](../param/07_tier.md), [`token::`](../param/08_token.md), [`expires::`](../param/09_expires.md), [`email::`](../param/10_email.md), [`file::`](../param/11_file.md), [`saved::`](../param/12_saved.md), [`display_name::`](../param/14_display_name.md), [`role::`](../param/15_role.md), [`billing::`](../param/16_billing.md), [`model::`](../param/17_model.md)
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
| `format::` | [`OutputFormat`](../type/02_output_format.md) | `text` | Output format |
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
# {"subscription":"max","tier":"default_claude_max_20x","token":"valid","expires_in_secs":26640,"email":"alice@acme.com","account":"alice@acme.com","file":"/home/user/.claude/.credentials.json","saved":2,"display_name":"alice","role":"admin","billing":"stripe_subscription","model":"sonnet"}
```

**Notes:**
- Field-presence params only affect text output. `format::json` always includes all fields regardless of field-presence params.
- `account::` reads the `_active` marker; shows `N/A` on machines where no account has ever been saved.
- `saved::` counts `*.credentials.json` files in the credential store; shows `0` when the credential store is absent.
