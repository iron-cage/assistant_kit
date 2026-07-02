# Runtime File: Version History Cache

### Scope

- **Purpose**: Document the on-disk cache storing GitHub Releases API responses for `.version.history`.
- **Responsibility**: Describe the cache file path, format, owner module, lifecycle triggers, and crash durability classification.
- **In Scope**: Cache path, TTL policy, JSON format, creation and refresh triggers, stale file handling.
- **Out of Scope**: GitHub API rate limits and network failure handling (→ `feature/001_version_management.md`).

### Abstract

Caches the GitHub Releases API response for the `anthropics/claude-code` repository to avoid redundant network calls. The cache expires after 1 hour; a stale or absent cache triggers a fresh API fetch and a new write. No clv command deletes this file.

### Path

`~/.claude/.transient/version_history_cache.json`

Resolution:
1. `$HOME/.claude/.transient/version_history_cache.json` — primary path
2. If `HOME` is unset, caching is skipped and the API is called directly on every invocation.

### Format

Raw JSON response body from the GitHub Releases API (`GET /repos/anthropics/claude-code/releases?per_page=100`). Top-level array of release objects. Written verbatim — no clv-specific schema wrapping.

### Owner

`src/commands/history.rs` — `fetch_releases_json()`. Creates `~/.claude/.transient/` via `std::fs::create_dir_all` on first write, then writes the cache file via `std::fs::write`.

### Lifecycle

- **Created:** On the first successful `.version.history` invocation when no cache file exists.
- **Refreshed:** On any `.version.history` invocation when the existing cache file's mtime age exceeds `CACHE_TTL_SECS` (3600 seconds).
- **Preserved:** Returned as-is when cache age is under 3600 seconds, without a network call.
- **Never deleted:** clv does not remove this file. It persists until the user cleans `~/.claude/.transient/`.

### Durability

**Classification:** safe-to-lose

A missing or corrupt cache file causes one additional GitHub API call on the next `.version.history` invocation; the file is then re-created. No permanent data loss occurs. Deletion is always safe.

### Features

| File | Relationship |
|------|-------------|
| [feature/001_version_management.md](../feature/001_version_management.md) | `.version.history` command that reads and writes this cache |
| [feature/008_runtime_file_discovery.md](../feature/008_runtime_file_discovery.md) | `.runtime_files` command that enumerates this path |
