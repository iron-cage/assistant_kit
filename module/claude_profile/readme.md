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
| `run/` | Container runner: thin wrapper, config manifest, and test script. |
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
- Account credential snapshots in `$PRO/.persistent/claude/credential/` (or `$HOME/.persistent/...`)
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
use claude_profile::{ account, token, ClaudePaths, PersistPaths };

// Where are the files?
let claude = ClaudePaths::new().expect( "HOME must be set" );
let persist = PersistPaths::new().expect( "HOME must be set" );
let credential_store = persist.credential_store();
println!( "credentials:      {}", claude.credentials_file().display() );
println!( "credential_store: {}", credential_store.display() );
println!( "projects:         {}", claude.projects_dir().display() );

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
for acct in account::list( &credential_store ).expect( "failed to list accounts" )
{
  let active = if acct.is_active { " ← active" } else { "" };
  println!( "{}{} ({})", acct.name, active, acct.subscription_type );
}

// Save current credentials as "work@acme.com"
account::save( "work@acme.com", &credential_store, &claude ).expect( "failed to save account" );

// Switch to "personal@home.com"
account::switch_account( "personal@home.com", &credential_store, &claude ).expect( "failed to switch" );

// Delete an old account
account::delete( "old@acme.com", &credential_store ).expect( "failed to delete" );
```

## File Paths

```rust,no_run
use claude_profile::{ ClaudePaths, PersistPaths };

let p = ClaudePaths::new().expect( "HOME must be set" );
let persist = PersistPaths::new().expect( "HOME must be set" );
println!( "credentials:      {}", p.credentials_file().display() );
println!( "credential_store: {}", persist.credential_store().display() );
println!( "projects:         {}", p.projects_dir().display() );
println!( "stats:            {}", p.stats_file().display() );
println!( "settings:         {}", p.settings_file().display() );
println!( "session-env:      {}", p.session_env_dir().display() );
println!( "sessions:         {}", p.sessions_dir().display() );
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
./run/docker .test
```

**Container (offline — no credentials needed):**
```bash
./run/docker .test.offline
```

**Container (interactive shell):**
```bash
./run/docker .shell
```

**Local (w3 required):**
```bash
./run/test
```
