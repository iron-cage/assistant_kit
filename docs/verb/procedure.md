# Verb Directory Operations

- **Actor:** Developer
- **Trigger:** A new module is added to the workspace and needs a `verb/` directory, OR an existing verb command needs updating across a module.
- **Emits:** —

## Add verb/ Directory to a New Module

1. Determine module type: **binary** (has a `[[bin]]` entry in `Cargo.toml`) or **library** (lib-only).
2. Create `module/<name>/verb/` directory.
3. Create `verb/build`: `cargo build -p <name>` (universal).
4. Create `verb/test`: `exec w3 .test level::3` (universal — identical across all modules).
5. Create `verb/clean`: `cargo clean -p <name>` (universal).
6. Create `verb/run`:
   - **Binary module:** `cargo run -p <name> --bin <binary> -- "$@"` with `--dry-run` printing the command.
   - **Library module:** `echo "verb 'run' is not available for this project" >&2; exit 3` (no `--dry-run`).
7. Create `verb/lint`: `cargo clippy -p <name> --all-features -- -D warnings` (universal).
8. Create `verb/verbs`: `printf` table with verb/status/command for all 7 verbs; library modules show `unavailable` for `run`.
9. Create `verb/detect`: fixed output — `ecosystem: cargo`, `signal: Cargo.toml`, `confidence: inferred` (universal — identical across all cargo modules).
10. Set executable bit on all 7 scripts: `chmod +x module/<name>/verb/*`.
11. If the module has runbox infrastructure: set `test_script: module/<name>/verb/test` in `module/<name>/run/runbox.yml` (see `docs/runbox/parameter/005_test_script.md`).
12. Add `| \`verb/\` | Shell scripts for each \`do\` protocol verb. |` row to `module/<name>/readme.md` Responsibility Table.

## Update a Verb Command

1. Identify the module and verb to change (e.g., `claude_profile/verb/build`).
2. Read the current script to understand what changes.
3. Edit the script: update the `exec` line and the matching `--dry-run` echo to stay in sync.
4. If the verb is `test`: also verify `runbox.yml` `test_script` still points to `verb/test` (it should — path does not change when command changes).
5. If the verb is `run` or `build`: update `verb/verbs` to reflect the new command string in the table.
6. Run `verb/test` (or `./run/docker .test`) to verify the module still passes.
