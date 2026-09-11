# Subcommand: plugin

Manage Claude Code plugins.

### Usage

```
claude plugin|plugins [command]
```

### Sub-subcommands

13 total (plus the generic commander.js `help [command]`, omitted as
boilerplate) — an earlier revision of this doc documented only 8 and omitted
`details`, `eval`, `init`/`new`, `prune`/`autoremove`, and `tag` entirely;
corrected against `claude plugin --help` on v2.1.220:

| Command | Description |
|---------|-------------|
| `details [options] <name>` | Show a plugin's component inventory and projected token cost |
| `disable [options] [plugin]` | Disable an enabled plugin |
| `enable [options] <plugin>` | Enable a disabled plugin |
| `eval [options] [command] [target]` | Run eval cases (`evals/**/case.yaml` or `evals/**/prompt.md` + `graders/*.md`) against a plugin and report scored results — its own `init` sub-subcommand authors a new eval suite |
| `init\|new [options] <name>` | Scaffold a new plugin at `~/.claude/skills/<name>/` (auto-loads next session as `<name>@skills-dir`) |
| `install\|i [options] <plugin>` | Install a plugin from available marketplaces (`plugin@marketplace` for specific) |
| `list [options]` | List installed plugins |
| `marketplace` | Manage Claude Code plugin marketplaces |
| `prune\|autoremove [options]` | Remove auto-installed dependencies that are no longer needed |
| `tag [options] [path]` | Create a `{name}--v{version}` git tag for a plugin release, validating that `plugin.json` and any enclosing marketplace entry agree |
| `uninstall\|remove [options] <plugin>` | Uninstall an installed plugin |
| `update [options] <plugin>` | Update a plugin to latest version (restart required) |
| `validate [options] <path>` | Validate a plugin or marketplace manifest |

### Description

Full plugin lifecycle management, substantially broader than install/enable/
list: it also covers authoring (`init`/`new` scaffolds a new plugin;
`eval init` authors a test suite for one), testing (`eval` runs scored
grader-based test cases with cost/threshold controls), release tooling
(`tag` cuts a validated git tag), maintenance (`prune`/`autoremove`), and
inspection (`details` reports token cost). Plugins extend Claude Code with
additional tools, MCP servers, and capabilities. They can be installed from
marketplaces, enabled/disabled individually, and validated for correctness.

The `marketplace` sub-subcommand manages the list of plugin sources. The
`validate` sub-subcommand checks a local plugin manifest for structural
correctness.

Alias: `claude plugins` works identically to `claude plugin`.

### Sub-subcommand Options

#### `claude plugin details`

No options beyond `-h`/`--help`.

#### `claude plugin eval`

| Option | Description |
|--------|-------------|
| `--ablation <mode>` | Run a no-plugin baseline arm and report the score delta (`none`\|`with-without`; default: `with-without` when targeting a plugin by name, `none` for a path) |
| `--allow-tools <tools...>` | Operator grant for gated tools (Bash, Write, Edit, WebFetch, `mcp__*`). Supports `Tool(pattern:*)` syntax |
| `--case <glob>` | Filter cases by name glob |
| `--json [path]` | Print the full run result as JSON to stdout, or write it to this `.json` file |
| `--judge-model <model>` | Override LLM-grader model (default: haiku) |
| `--keep-temp` | Preserve scaffold dirs for debugging |
| `--max-cost-usd <usd>` | Hard cost ceiling; abort and report partial results if hit (exit 2) |
| `--model <model>` | Override model for all cases |
| `--no-scaffold` | Explicitly skip `scaffold_script` |
| `--output-dir <dir>` | Directory for `aggregate-result.json` (default: `./evals/results/<timestamp>/`) |
| `--publish-report` | Publish the HTML report privately to claude.ai and print its link |
| `--report <path>` | Write a self-contained HTML report to `<path>` |
| `--runs <n>` | Override per-case runs (default: `case.runs ?? 3`) |
| `--scaffold` | Run each case's `scaffold_script` (runs author-supplied bash as you; off by default) |
| `--tag <tag...>` | Filter cases by tag (repeatable) |
| `--threshold <0..1>` | Exit 1 if any case score is below this threshold (default: 1.0) |
| `--verbose` | Stream the trace as it runs |

Its own nested sub-subcommand: `eval init [options] [name]` — authors a new
eval suite under `evals/` via an interview; `--bare <name>` scaffolds a blank
single-case template instead.

#### `claude plugin init` (alias `new`)

| Option | Description |
|--------|-------------|
| `--author <name>` | Author name (default: `git config user.name`) |
| `--author-email <email>` | Author email (default: `git config user.email`) |
| `--description <text>` | Manifest description |
| `-f`, `--force` | Overwrite an existing `.claude-plugin/` at the target |
| `--with <components...>` | Also scaffold: `skills`, `agents`, `hooks`, `mcp`, `lsp`, `output-style`, `channel` |

#### `claude plugin prune` (alias `autoremove`)

| Option | Description |
|--------|-------------|
| `--dry-run` | List what would be removed without removing |
| `-s`, `--scope <scope>` | Prune at scope: `user`, `project`, or `local` (default: `user`) |
| `-y`, `--yes` | Skip the confirmation prompt (required when stdin/stdout is not a TTY) |

#### `claude plugin tag`

| Option | Description |
|--------|-------------|
| `--dry-run` | Print what would be tagged without creating it |
| `-f`, `--force` | Skip the dirty-working-tree and tag-already-exists checks |
| `-m`, `--message <msg>` | Tag annotation message (use `%s` for the version) |
| `--push` | Push the tag to `--remote` after creating it |
| `--remote <name>` | Remote to push to with `--push` (default: `origin`) |

#### `claude plugin install`

| Option | Description |
|--------|-------------|
| `-s`, `--scope <scope>` | Installation scope: `user`, `project`, or `local` (default: user) |

#### `claude plugin uninstall`

| Option | Description |
|--------|-------------|
| `-s`, `--scope <scope>` | Uninstall from scope: `user`, `project`, or `local` (default: user) |

#### `claude plugin list`

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |
| `--available` | Include available plugins from marketplaces (requires `--json`) |

#### `claude plugin enable`

| Option | Description |
|--------|-------------|
| `-s`, `--scope <scope>` | Installation scope: `user`, `project`, `local` (default: auto-detect) |

#### `claude plugin disable`

| Option | Description |
|--------|-------------|
| `-a`, `--all` | Disable all enabled plugins |
| `-s`, `--scope <scope>` | Installation scope: `user`, `project`, `local` (default: auto-detect) |

#### `claude plugin update`

| Option | Description |
|--------|-------------|
| `-s`, `--scope <scope>` | Installation scope: `user`, `project`, `local`, `managed` (default: user) |

#### `claude plugin marketplace add`

| Option | Description |
|--------|-------------|
| `--scope <scope>` | Config scope: `user`, `project`, or `local` |
| `--sparse <paths...>` | Git sparse-checkout paths (for monorepos) |

#### `claude plugin marketplace list`

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |

### Since

v2.0.12 (2025-10-09) for the subcommand family in general. `details`, `eval`,
`init`/`new`, `prune`/`autoremove`, and `tag` are not dated separately in the
`version/` collection — all are present in v2.1.220.

### Verification

```bash
claude plugin --help    # → 13 sub-subcommands + help
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [../param/024_enabled_plugins.md](../param/024_enabled_plugins.md) | `enabledPlugins` config key |
| doc | [../param/048_plugin_dir.md](../param/048_plugin_dir.md) | `--plugin-dir` parameter |
| doc | [../param/088_plugin_prefer_https.md](../param/088_plugin_prefer_https.md) | Plugin HTTPS preference |
