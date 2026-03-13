# module

| Crate | Responsibility |
|-------|---------------|
| claude_common | Layer 0 shared primitives: ClaudePaths and process utilities |
| claude_profile_core | Layer 1 domain logic: token status and account management |
| claude_runner | CLI binary for executing Claude Code |
| claude_runner_core | Core library for spawning Claude Code process |
| claude_profile | Account credential management, token status, path topology |
| claude_storage | CLI tool for Claude Code storage exploration |
| claude_storage_core | Zero-dep core library for Claude storage access |
| claude_manager | CLI for managing Claude Code installation and lifecycle |
| claude_manager_core | Layer 1 domain logic: version, session, settings, account |
| claude_tools | Layer 3 super-app aggregating all claude_* CLI commands into clt |
