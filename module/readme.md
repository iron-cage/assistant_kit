# module

| Crate | Responsibility |
|-------|---------------|
| claude_assets_core | Layer 1 domain logic: symlink-based Claude Code artifact installer |
| claude_assets | CLI for installing Claude Code artifacts via symlinks (cla binary) |
| claude_core | Layer 0 shared primitives: ClaudePaths and process utilities |
| claude_profile_core | Layer 1 domain logic: token status and account management |
| claude_runner | CLI binary for executing Claude Code |
| claude_runner_core | Core library for spawning Claude Code process |
| claude_profile | Account credential management, token status, path topology |
| claude_storage | CLI tool for Claude Code storage exploration |
| claude_storage_core | Zero-dep core library for Claude storage access |
| claude_version | Claude Code version manager CLI |
| claude_version_core | Layer 1 domain logic: version, session, settings, account |
| claude_auth | Layer * standalone primitive: Anthropic OAuth token refresh transport |
| claude_quota | Layer * standalone primitive: Anthropic API rate-limit HTTP transport |
| dream | Layer 2 library facade re-exporting all core crates (Layer 0, *, 1) |
| assistant | Layer 3 super-app aggregating all Layer 2 CLI tools into ast |
