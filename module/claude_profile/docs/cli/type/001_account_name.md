# Type :: 1. `AccountName`

**Purpose:** Identifies a credential profile in the account store using the account's email address as the unique key. Enforces email format to guarantee correct identification and safe file creation under the credential store.

**Fundamental Type:** Newtype wrapping `String`

```rust
pub struct AccountName( String );
```

**Constants:**
- No predefined constants — all valid names are user-defined

**Constraints:**
- Non-empty (reject `""`)
- Must contain `@` with non-empty local part and domain (valid email format)
- Local part (before `@`) must not contain `/`, `\`, or `*` (path-unsafe chars rejected before any filesystem operation)
- Maps to file `{credential_store}/{email}.credentials.json`

**Parsing:**

```rust
impl AccountName
{
  pub fn new( s : &str ) -> Result< Self, String >
  {
    if s.is_empty() { return Err( "account name must not be empty".into() ); }
    let at = s.find( '@' ).ok_or_else( || format!( "account name '{}' must be an email address", s ) )?;
    if at == 0 || at == s.len() - 1
    {
      return Err( format!( "account name '{}' must be an email address", s ) );
    }
    Ok( Self( s.to_string() ) )
  }
}
```

**Methods:**
- `get() -> &str` — raw string accessor
- `to_credential_path( credential_store : &Path ) -> PathBuf` — resolves `{credential_store}/{name}.credentials.json`

**Parameters:** [`name::`](../param/001_name.md)

**Commands:** [`.accounts`](../command/001_account.md#command--3-accounts) *(optional)*, [`.account.save`](../command/001_account.md#command--4-accountsave), [`.account.use`](../command/001_account.md#command--5-accountuse), [`.account.delete`](../command/001_account.md#command--6-accountdelete), [`.account.limits`](../command/001_account.md#command--11-accountlimits) *(optional)*

**Notes:**
- `AccountName` is the post-resolution type — it always holds a validated email.
- Pre-resolution (before the adapter resolves prefix/positional forms) is handled by [`AccountSelector`](004_account_selector.md).
