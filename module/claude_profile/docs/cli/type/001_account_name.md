# Type :: 1. `AccountName`

**Purpose:** Identifies a credential profile in the account store using the account's email address as the unique key. Enforces email format to guarantee correct identification and safe file creation under the credential store.

**Fundamental Type:** Newtype wrapping `String`

**Constants:**
- No predefined constants — all valid names are user-defined

**Constraints:**
- Non-empty (reject `""`)
- Must contain `@` with non-empty local part and domain (valid email format)
- Local part (before `@`) must not contain `/`, `\`, or `*` (path-unsafe chars rejected before any filesystem operation)
- Maps to file `{credential_store}/{email}.credentials.json`

**Parsing:**

```
pub fn new( s : &str ) -> Result< Self, String >
```

**Methods:**
- `get() -> &str` — raw string accessor
- `to_credential_path( credential_store : &Path ) -> PathBuf` — resolves `{credential_store}/{name}.credentials.json`

**Notes:**
- `AccountName` is the post-resolution type — it always holds a validated email.
- Pre-resolution (before the adapter resolves prefix/positional forms) is handled by [`AccountSelector`](004_account_selector.md).

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`name::`](../param/001_name.md) | Accepts user-supplied account identifier |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md#command--3-accounts) | Optional — account list display |
| 2 | [`.account.save`](../command/001_account.md#command--4-accountsave) | Persists credential profile keyed by name |
| 3 | [`.account.use`](../command/001_account.md#command--5-accountuse) | Activates saved account |
| 4 | [`.account.delete`](../command/001_account.md#command--6-accountdelete) | Removes credential profile |
| 5 | [`.account.limits`](../command/001_account.md#command--11-accountlimits) | Optional — account limits display |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Active account resolved by name |
| 2 | [Account Onboarding](../user_story/002_onboarding.md) | Name identifies profiles managed |
