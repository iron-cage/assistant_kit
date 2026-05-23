# Parameter :: 16. `billing::`

Controls whether the billing type line appears in output. Opt-in (default `0`). Source: `billingType` field in `oauthAccount` — read from live `~/.claude.json` (`.credentials.status`) or from the saved `{name}.claude.json` snapshot (`.accounts`).

- **Type:** `bool`
- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](../command/account.md#command--3-accounts), [`.credentials.status`](../command/credentials.md#command--10-credentialsstatus)
- **Purpose:** Shows the raw billing type string (e.g., `stripe_subscription`). Shows `N/A` when the source file is absent or the field is missing.
- **Group:** [Field Presence](../param_group/02_field_presence.md)

**Examples:**

```text
billing::0   → line omitted  (default)
billing::1   → Billing: stripe_subscription
```
