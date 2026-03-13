# claude_profile — Vision

Claude Code has an internal file system that nobody documents. Run it long enough and you
start noticing the shape of things: conversations accumulate in `~/.claude/projects/`
organised by an escaped working-directory path, credentials live in a JSON file with a
specific shape, session environment records sit in their own subdirectory, usage statistics
cache next to settings. The structure is consistent, reproducible, and entirely undocumented.

`claude_profile` turns that implicit structure into an explicit, typed, tested Rust API.
That is its founding purpose, and everything else follows from it.

## The Filesystem It Owns

```
~/.claude/
  .credentials.json             ← active OAuth token; shape is the API contract
  accounts/
    work.credentials.json       ← named credential snapshot
    personal.credentials.json
    _active                     ← "work"  ← single text file, single responsibility
  projects/
    -home-user-project-/        ← path-escaped working dir
      abc123.jsonl              ← conversation history
      sessions-index.json
  session-env/{uuid}/           ← per-invocation environment records
  sessions/{id}.json            ← session records
  stats-cache.json
  settings.json
```

Every path in that tree that matters to tooling is computed through
[`ClaudePaths`](src/paths.rs). Nothing in this codebase hardcodes `~/.claude/` as a
string literal — not once. `ClaudePaths` is the single authoritative source and every
other module that needs a path calls it.

## The Primary Problem: Account Rotation

Claude Code subscriptions enforce a 5-hour active-use window per cycle. Developers with
multiple subscriptions — work, personal, a team seat — need to rotate between them to
maintain uninterrupted availability. Before this crate, doing that meant opening a text
editor and manually replacing the contents of `.credentials.json`. One mistimed write,
one interrupted save, and the credential file is half-formed. Claude Code will not start.

The entire solution turns out to be a single atomic file operation.

```rust
// The full implementation of account switching is:
std::fs::copy( &src, &tmp )?;           // stage to adjacent .json.tmp
std::fs::rename( &tmp, &credentials )?; // atomically replace
std::fs::write( &marker, name )?;       // update _active
```

Both files share `~/.claude/` — the same filesystem — which guarantees the rename is
atomic. A crash at any point leaves either the old credentials or the new ones, never a
half-written file. This turns a dangerous manual operation into a safe, one-call API:

```rust
claude_profile::account::switch_account( "personal" )?;
```

Account save, list, and delete complete the lifecycle. Save is a copy. Delete is a remove
with one guard: you cannot delete the active account. The guard prevents the credential
pointer from dangling, which would make the next switch succeed but point at nothing.

## A Distinction That Matters

One correctness point that took deliberate thought: `expiresAt` in `.credentials.json`
is the **OAuth access token expiry** — typically months out, auto-refreshed by Claude Code
before it lapses. It is **not** the 5-hour subscription usage window, which is server-side
state that has no local representation.

`TokenStatus` therefore classifies the OAuth token, not the usage window:

```rust
pub enum TokenStatus {
    Valid { expires_in: Duration },         // more than 60 min remaining
    ExpiringSoon { expires_in: Duration },  // within 60 min — consider rotating
    Expired,                                // OAuth token is past expiresAt
}
```

This distinction matters in automation. A `Valid` token still means you may have exhausted
your subscription window — that's a different signal, observable only by attempting a
request and seeing the server's response. Getting this wrong would mean automation that
appears to work (token looks valid) but silently fails (window exhausted). Getting it right
means the crate makes no claim it cannot verify.

## The Boundary That Is a Feature

`claude_profile` MUST NOT execute any processes. `std::process::Command` is not imported
anywhere in `src/`. If it ever appears, the test suite fails immediately:

```
RESPONSIBILITY VIOLATION: claude_profile MUST NOT import std::process::Command
```

This is not a convention. It is a static analysis test that runs on every commit.

Why does this matter? Because `claude_profile` holds the credential path. A crate that
knows where `~/.claude/.credentials.json` lives and also spawns subprocesses creates a
surface that mixes data access with execution — the kind of surface that, once established,
tends to grow. Keeping them strictly separated means the credential-touching code is
auditable in complete isolation. The execution crate (`claude_runner_core`) knows nothing
about credentials. The credential crate knows nothing about execution.

## Where the Crate Stands

All four library modules — `account`, `token`, `paths`, `persist` — are implemented and
fully tested. The CLI binary layer adds `adapter`, `output`, and `commands` (9 commands
behind the `enabled` feature). Functional requirements live in [`docs/feature/`](docs/feature/) (FR-6 through FR-18);
each doc instance maps to named tests via Cross-References sections. All tests pass. No clippy warnings.

The crate is used in production for path resolution and account rotation.

## The One-Liner (FR-13: Auto Rotate)

Every rotation workflow built on this crate needed the same boilerplate: list accounts,
filter out the active one, pick the candidate with the highest `expires_at_ms`, switch.
That decision logic now lives in the library, not in every caller:

```rust
// Before (repeated in every caller):
let candidate = account::list()?
    .into_iter()
    .filter( | a | !a.is_active )
    .max_by_key( | a | a.expires_at_ms )
    .ok_or_else( || /* no accounts available */ )?;
account::switch_account( &candidate.name )?;

// After (FR-13):
account::auto_rotate()?;  // returns the name of the account switched to
```

`auto_rotate()` is FR-13, implemented in `account.rs`. It completes the
"seamless rotation" promise and makes the primary use case a one-liner.
