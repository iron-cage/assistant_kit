# claude_profile

Claude Code account credential management.

## Files

| File / Directory | Responsibility |
|------------------|----------------|
| `Cargo.toml` | Crate manifest: dependencies, features, metadata |
| `src/` | Library modules and CLI binary (account, token, paths, adapter, commands) |
| `tests/` | Test suite for credential management |
| `docs/` | Behavioral requirements: features (FR-6–FR-18), invariants, CLI reference |
| `unilang.commands.yaml` | YAML command metadata for 10 profile commands |
| `Dockerfile` | Container image definition for isolated test runs. |
| `scripts/` | Container CLI for building and running tests in isolation. |
| `vision.md` | Crate vision, design decisions, and open problems |
| `vision_ua.md` | Crate vision in Ukrainian |
| `changelog.md` | Notable changes by version |

### Responsibility Table

| Entity | Responsibility | Input→Output | Scope | Out of Scope |
|--------|---------------|--------------|-------|--------------|
| `account` | Named credential storage and rotation | name → active credentials | Save, list, switch, delete accounts | ❌ OAuth HTTP refresh → network dep<br>❌ Browser launch → caller |
| `token` | Active OAuth token expiry status | credentials file → `TokenStatus` | Read `expiresAt`, classify Valid/ExpiringSoon/Expired | ❌ Token refresh → HTTP<br>❌ Server-side window → unobservable |
| `paths` | `~/.claude/` file topology | `HOME` → canonical `PathBuf`s | All `~/.claude/` path constants | ❌ Process execution |
| `persist` | Persistent user storage path resolution | `$PRO`/`$HOME` → `PathBuf` | Resolve `$PRO/persistent/claude_profile/` with `$HOME` fallback (FR-15) | ❌ Writing data → caller |

### Scope

**In Scope:**
- Account credential snapshots in `~/.claude/accounts/`
- Token expiry detection from `~/.claude/.credentials.json`
- All canonical `~/.claude/` paths via `ClaudePaths`
- Persistent user storage path resolution via `PersistPaths` (`$PRO`/`$HOME`)

**Out of Scope:**
- ❌ Claude Code process execution → `claude_runner_core`
- ❌ Continuation detection (session file existence) → `claude_storage_core`
- ❌ Session directory management → `claude_runner_core::SessionManager`
- ❌ Pulse keeping (periodic `claude` invocation) → caller + `claude_runner`
- ❌ Browser launch / `xdg-open` → caller
- ❌ OAuth HTTP token refresh → network dependency not allowed
- ❌ Server-side 5-hour subscription window → not locally observable

## Account Management

```rust,no_run
use claude_profile::{ account, token, ClaudePaths };

// Where are the files?
let p = ClaudePaths::new().expect( "HOME must be set" );
println!( "credentials: {}", p.credentials_file().display() );
println!( "accounts:    {}", p.accounts_dir().display() );
println!( "projects:    {}", p.projects_dir().display() );

// Check active token status
match token::status().expect( "failed to read credentials" )
{
  token::TokenStatus::Valid { expires_in } =>
    println!( "ok — {}m remaining", expires_in.as_secs() / 60 ),
  token::TokenStatus::ExpiringSoon { expires_in } =>
    eprintln!( "expires in {}m — consider switching accounts", expires_in.as_secs() / 60 ),
  token::TokenStatus::Expired =>
    eprintln!( "token expired — run: claude auth login" ),
}

// List all stored accounts
for acct in account::list().expect( "failed to list accounts" )
{
  let active = if acct.is_active { " ← active" } else { "" };
  println!( "{}{} ({})", acct.name, active, acct.subscription_type );
}

// Save current credentials as "work"
account::save( "work" ).expect( "failed to save account" );

// Switch to "personal"
account::switch_account( "personal" ).expect( "failed to switch" );

// Delete an old account
account::delete( "old-account" ).expect( "failed to delete" );
```

## File Paths

```rust,no_run
use claude_profile::ClaudePaths;

let p = ClaudePaths::new().expect( "HOME must be set" );
println!( "credentials: {}", p.credentials_file().display() );
println!( "accounts:    {}", p.accounts_dir().display() );
println!( "projects:    {}", p.projects_dir().display() );
println!( "stats:       {}", p.stats_file().display() );
println!( "settings:    {}", p.settings_file().display() );
println!( "session-env: {}", p.session_env_dir().display() );
println!( "sessions:    {}", p.sessions_dir().display() );
```

## Binary

Two names, same binary — both `claude_profile` and `clp` are installed:

```bash
clp .account.list          # list saved accounts
clp .usage                 # token usage statistics
clp .paths                 # show ~/.claude/ canonical paths
```

## Testing

**Container (all tests — credentials required):**
```bash
./scripts/docker .test
```

**Container (offline — no credentials needed):**
```bash
./scripts/docker .test.offline
```
