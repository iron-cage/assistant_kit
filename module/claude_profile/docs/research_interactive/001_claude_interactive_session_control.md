# Research: Controlling the Claude Interactive Session

### Scope

- **Purpose**: Document `claude` binary execution modes, authentication subcommands, and flag inventory from live research.
- **Responsibility**: Covers `claude auth login` discovery, `--print` mode constraints, environment variable controls, and implications for `.account.relogin`.
- **In Scope**: Flag inventory (2026-05-26); auth subcommands; execution modes 1–4; credential write behavior; HOME override notes.
- **Out of Scope**: Behavioral requirements (→ feature/019); test specifications.

- **Date:** 2026-05-26
- **Trigger:** User question — can `.account.relogin` avoid dropping into the full interactive REPL?
- **Method:** `claude --help` flag inventory + `claude auth --help` + `claude auth login --help` + `claude auth status --json` live run + source analysis of `ClaudeCommand` builder and `run_isolated()`

### Background

`.account.relogin` currently uses `execute_interactive()`, which calls `cmd.status()` (not `cmd.output()`) to attach the user's TTY to the subprocess. This drops the user into the full Claude REPL. The user must:

1. Complete browser login within the session
2. Manually type `/exit` or `Ctrl-C` to exit the REPL
3. Only then does the credential-change detection run

The question was whether we could trigger browser OAuth without entering the REPL at all.

### Key Discovery: `claude auth login`

`claude auth` is a dedicated authentication management subcommand. It was not previously documented in this codebase.

```
claude auth login [options]   Sign in to your Anthropic account

Options:
  --email <email>   Pre-populate email address on the login page
  --sso             Force SSO login flow
  -h, --help        Display help for command
```

**`claude auth login` is auth-only** — it opens the browser for OAuth and exits when done. No REPL, no `/exit` required. This is the correct tool for the relogin use case.

Additional `auth` subcommands:

```
claude auth status [--json|--text]   Show authentication status
claude auth logout                   Log out from current account
```

`claude auth status --json` output (live sample):
```json
{
  "loggedIn": true,
  "authMethod": "claude.ai",
  "apiProvider": "firstParty",
  "email": "i1@wbox.pro",
  "orgId": "c6b06228-c923-4834-9b7d-13002bea4dc1",
  "orgName": "i1@wbox.pro's Organization",
  "subscriptionType": "max"
}
```

Fields: `loggedIn` (bool), `authMethod` ("claude.ai" | "apiKey" | unknown), `apiProvider` ("firstParty" | unknown), `email`, `orgId`, `orgName`, `subscriptionType`.

### Full Flag Inventory (`claude --help`, 2026-05-26)

#### Execution mode flags

| Flag | Effect |
|------|--------|
| `-p` / `--print` | Non-interactive: print response and exit. Only mode that works in piped/non-TTY context. |
| (none) | Default: interactive REPL. Requires TTY. |

#### Session control

| Flag | Effect |
|------|--------|
| `-c` / `--continue` | Resume most recent conversation in current dir |
| `-r` / `--resume [id]` | Resume by session ID or open interactive picker |
| `--fork-session` | New session ID when resuming (use with `-c`/`-r`) |
| `--session-id <uuid>` | Specify session UUID explicitly |
| `--no-session-persistence` | Disable session persistence (only with `--print`) |

#### Output format (only work with `--print`)

| Flag | Effect |
|------|--------|
| `--output-format text\|json\|stream-json` | Output encoding |
| `--input-format text\|stream-json` | Input encoding |
| `--include-partial-messages` | Stream partial chunks (with `stream-json`) |
| `--replay-user-messages` | Re-emit user messages on stdout (with `stream-json`) |

#### Model / effort

| Flag | Effect |
|------|--------|
| `--model <model>` | Model alias (`sonnet`, `opus`) or full ID |
| `--effort <level>` | low, medium, high, max |
| `--fallback-model <model>` | Auto-fallback on overload (only with `--print`) |
| `--max-budget-usd <amount>` | Spending cap (only with `--print`) |

#### Permissions / security

| Flag | Effect |
|------|--------|
| `--dangerously-skip-permissions` | Bypass all permission checks (default ON for automation) |
| `--allow-dangerously-skip-permissions` | Make skip-permissions available as an option but not default |
| `--permission-mode <mode>` | acceptEdits, bypassPermissions, default, dontAsk, plan, auto |
| `--allowedTools <tools>` | Whitelist specific tools |
| `--disallowedTools <tools>` | Blacklist specific tools |
| `--tools <tools>` | Explicit tool list; `""` disables all |

#### Prompting / context

| Flag | Effect |
|------|--------|
| `--system-prompt <prompt>` | Override system prompt |
| `--append-system-prompt <prompt>` | Append to default system prompt |
| `--json-schema <schema>` | Structured output validation |
| `--file <specs>` | File resources to download at startup |
| `--add-dir <dirs>` | Additional directories for tool access |

#### IDE / browser / worktree

| Flag | Effect |
|------|--------|
| `--chrome` / `--no-chrome` | Claude in Chrome integration |
| `--ide` | Auto-connect to IDE |
| `--tmux` | Create tmux session (with `--worktree`) |
| `-w` / `--worktree [name]` | Create git worktree for session |

#### MCP / plugins

| Flag | Effect |
|------|--------|
| `--mcp-config <configs>` | Load MCP servers from JSON files or strings |
| `--strict-mcp-config` | Only use `--mcp-config` servers, ignore all others |
| `--plugin-dir <paths>` | Load plugins from directories |

#### Agents

| Flag | Effect |
|------|--------|
| `--agent <agent>` | Agent for current session |
| `--agents <json>` | JSON object defining custom agents |
| `--brief` | Enable `SendUserMessage` tool for agent-to-user communication |

#### Diagnostics

| Flag | Effect |
|------|--------|
| `-d` / `--debug [filter]` | Debug mode with optional category filter |
| `--debug-file <path>` | Write debug logs to file |
| `--verbose` | Override verbose mode from config |

#### Environment variables (from `ClaudeCommand` source)

| Variable | Effect |
|----------|--------|
| `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | Max output tokens (default in builder: 200,000) |
| `CLAUDE_CODE_BASH_TIMEOUT` | Bash tool timeout ms (default: 3,600,000) |
| `CLAUDE_CODE_BASH_MAX_TIMEOUT` | Bash max timeout ms (default: 7,200,000) |
| `CLAUDE_CODE_AUTO_CONTINUE` | Auto-continue conversations (default: true) |
| `CLAUDE_CODE_TELEMETRY` | Telemetry (default in builder: false) |
| `CLAUDE_CODE_AUTO_APPROVE_TOOLS` | Auto-approve tool calls |
| `CLAUDE_CODE_ACTION_MODE` | Action mode (acceptEdits/bypassPermissions/etc.) |
| `CLAUDE_CODE_LOG_LEVEL` | Log level |
| `CLAUDE_CODE_TEMPERATURE` | Temperature |
| `CLAUDE_CODE_SANDBOX_MODE` | Sandbox mode |
| `CLAUDE_CODE_SESSION_DIR` | Session storage directory |
| `CLAUDE_CODE_TOP_P` | Top-p sampling |
| `CLAUDE_CODE_TOP_K` | Top-k sampling |
| `HOME` | Override home dir (used by `run_isolated()`) |
| `CLAUDECODE` | Removed by builder (`unset_claudecode=true`) to prevent nested session detection |

#### Top-level commands

| Command | Effect |
|---------|--------|
| `claude auth login [--email] [--sso]` | Browser OAuth — auth-only, exits after login |
| `claude auth logout` | Log out current account |
| `claude auth status [--json\|--text]` | Query current auth state |
| `claude setup-token` | Set up long-lived auth token |
| `claude agents` | List configured agents |
| `claude mcp` | Configure MCP servers |
| `claude doctor` | Check auto-updater health |
| `claude install [target]` | Install specific version |
| `claude update` | Update to latest |

### Execution Mode Analysis

#### Mode 1: Full Interactive REPL (current `execute_interactive()`)

```rust
cmd.status()  // inherited TTY, stdout/stderr not captured
```

- Claude owns the terminal
- User sees full REPL prompt
- Suitable for: general-purpose interactive use, debugging
- **Problem for relogin:** user must manually `/exit` after auth

#### Mode 2: Non-interactive / headless (`execute()` with `--print`)

```rust
cmd.output()  // stdout/stderr captured, no TTY
```

With args `["--print", "."]`:
- Claude starts, attempts OAuth refresh at startup
- If refreshToken is **valid** → refresh succeeds, credentials file updated, exits
- If refreshToken is **dead** → refresh fails; in `--print` mode Claude exits with error or empty output — **does NOT open browser**
- Suitable for: token refresh (`refresh::1`), session touch (`touch::1`)
- **Not suitable for relogin:** cannot trigger browser OAuth in non-TTY mode

#### Mode 3: Auth-only (`claude auth login`) — **newly discovered**

```bash
claude auth login [--email <email>]
```

- Opens browser for OAuth login flow
- Process exits automatically when authentication completes or user cancels
- No REPL prompt
- No manual `/exit` required
- Suitable for: **exactly the relogin use case**
- Needs TTY for browser redirect/display, but exits cleanly after auth

#### Mode 4: Auth status query (`claude auth status --json`)

```bash
claude auth status --json
```

- Non-interactive
- Returns JSON: `loggedIn`, `authMethod`, `email`, `orgId`, `subscriptionType`
- Can be used to verify auth state pre/post relogin
- Could replace `aq.result = Err(...)` detection as a pre-flight check

### Implication for `.account.relogin`

Current code in `account_relogin_routine()` (`commands.rs:1209`):
```rust
let spawn_result = claude_runner_core::ClaudeCommand::new()
  .execute_interactive();
```

Proposed replacement:
```rust
// Use auth login subcommand — avoids full REPL; exits automatically after browser auth.
let spawn_result = claude_runner_core::ClaudeCommand::new()
  .with_chrome( None )           // auth login doesn't need --chrome
  .with_args( [ "auth", "login", "--email", name.as_str() ] )
  .execute_interactive();        // still needs TTY for browser redirect
```

Benefits:
1. **No manual `/exit`** — `claude auth login` exits automatically after auth
2. **Pre-populated email** — `--email <name>` suggests the correct account on the browser login page
3. **Scoped intent** — the subprocess is auth-only, not a general REPL
4. **Same credential detection** — `auth login` writes to `~/.claude/.credentials.json` like the REPL does

Risks / open questions:
1. **HOME override** — does `claude auth login` respect a custom `HOME` env var? Must verify credentials are written to `~/.claude/.credentials.json` in the overridden home.
2. **SSO accounts** — `--sso` flag may be needed for enterprise accounts; should be surfaced via a `sso::` param on `.account.relogin`.
3. **Credentials format** — does `auth login` produce the same credential JSON structure as the REPL? (Assumed yes — same underlying OAuth flow.)
4. **Exit code on cancel** — what exit code does `auth login` return when the user cancels? Behavior needs testing.
5. **Pre-spawn message** — regardless of approach, a message should be emitted before the subprocess so the user knows what's happening.

### Additional Improvement: Pre-spawn Diagnostic Message

Currently `account_relogin_routine()` emits zero output before calling `execute_interactive()`. The user is silently dropped into a subprocess. The workflow docs show expected output like:

```
[relogin] spawning claude for browser re-authentication (Ctrl-C to abort)
```

But this `eprintln!` call doesn't exist in the source (`commands.rs:1209`). This is a UX gap independent of which subprocess approach is used.

### Recommendation

File two tasks:
1. **UX fix (easy):** Add `eprintln!` before the subprocess call in `account_relogin_routine()` — tell user what's happening before auth starts.
2. **Auth approach improvement (medium):** Replace `ClaudeCommand::new().execute_interactive()` with `ClaudeCommand::new().with_args(["auth", "login", "--email", name]).execute_interactive()` — targeted auth-only subprocess. Validate credential write behavior before implementing.

Also consider:
3. **`sso::` param** on `.account.relogin` for enterprise SSO flows.
4. **`claude auth status --json`** as a pre-flight check before deciding relogin is needed (could be integrated into `clp .account.limits` or `.usage` error path).
