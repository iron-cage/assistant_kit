# Verb Directory Operations

- **Actor:** Developer
- **Trigger:** A new module is added to the workspace and needs a `verb/` directory, OR an existing verb command needs updating across a module.
- **Emits:** —

## Add verb/ Directory to a New Module

1. Determine module type: **binary** (has a `[[bin]]` entry in `Cargo.toml`) or **library** (lib-only).
2. Create `module/<name>/verb/` directory.
3. Create `verb/build`: `cargo build -p <name>` (universal).
4. Create `verb/test` dispatcher + `verb/test.d/l1` (`w3 .test level::3`). Both are universal — identical across all cargo modules.
5. Create `verb/clean`: `cargo clean -p <name>` (universal).
6. Create `verb/run`:
   - **Binary module:** dispatcher at `verb/run` + `verb/run.d/l1` (`cargo run -p <name> --bin <binary>`) with `--dry-run` support in l1.
   - **Library module:** `echo "verb 'run' is not available for this project" >&2; exit 3` in `verb/run` (no layers needed).
7. Create `verb/lint` dispatcher + `verb/lint.d/l1` (`cargo clippy -p <name> --all-features -- -D warnings`). Both universal — identical dispatcher, module-specific l1.
8. Create `verb/verify`: `exec w3 .test level::4` (universal — identical across all modules); add `--dry-run` echo printing `w3 .test level::4`.
9. Create `verb/verbs`: `printf '%-13s ...'` table with verb/status/command for all 8 verbs; library modules show `unavailable` for `run`; last row is `package_info built-in  -`.
10. Create `verb/package_info`: Python3 script — reads `name`, `version`, `edition` from `Cargo.toml` (resolves workspace inheritance via `../../Cargo.toml`), static fields for `language`/`package_manager`/`signal`/`confidence`; prints deterministic flat JSON with blank line before `{` and after `}` (universal — identical across all cargo modules).
11. Set executable bit: `chmod +x module/<name>/verb/*` (dispatchers + plain scripts) and `chmod +x module/<name>/verb/*.d/*` (layer files).
12. If the module has runbox infrastructure: set `test_script: module/<name>/verb/test` in `module/<name>/run/runbox.yml` (see `run/docs/parameter/005_test_script.md`).
13. Add `| \`verb/\` | Shell scripts for each \`do\` protocol verb. |` row to `module/<name>/readme.md` Responsibility Table.

## Add verb/ Directory to a Standalone Runbox Project

For non-Rust standalone projects (Python, Node.js, etc.) the `verb/` scripts delegate to the container runner instead of calling cargo.

1. Determine project type: **binary** (has entry point) or **library** (no runnable entry point).
2. Create `verb/build`: `exec ./run/runbox .build` (universal).
3. Create `verb/test` dispatcher + `verb/test.d/l1` (ecosystem runner) + `verb/test.d/l2` (`./run/runbox .test`). Set `test_script: verb/test` in `runbox.yml`.
4. Create `verb/clean`: remove build artifacts specific to the ecosystem (`.venv/`, `node_modules/`, `target/`).
5. Create `verb/run`:
   - **Binary project:** dispatcher at `verb/run` + `verb/run.d/l1` (ecosystem entry point) + `verb/run.d/l2` (`./run/runbox .run`); set `run_script: verb/run` in `runbox.yml`.
   - **Library project:** `echo "verb 'run' is not available for this project" >&2; exit 3` (no layers needed).
6. Create `verb/lint` dispatcher + `verb/lint.d/l1` (ecosystem linter) + `verb/lint.d/l2` (`./run/runbox .lint`). Set `lint_script: verb/lint` in `runbox.yml`.
7. Create `verb/verify`: `exec ./run/runbox .test` (same as test — no level::4 concept outside Rust).
8. Create `verb/verbs`: same `printf` table format as Rust; ecosystem-specific commands in the table.
9. Create `verb/package_info`: reads the ecosystem manifest (`pyproject.toml`, `package.json`, `Cargo.toml`) and emits flat JSON. Match the field set used by Rust projects.
10. Set executable bit: `chmod +x verb/*` (dispatchers + plain scripts) and `chmod +x verb/*.d/*` (layer files).
11. Add `| \`verb/\` | Shell scripts for each \`do\` protocol verb. |` row to project `readme.md`.

## Update a Verb Command

1. Identify the module and verb to change (e.g., `claude_profile/verb/build`).
2. Read the current script to understand what changes.
3. Edit the layer script (`verb/<verb>.d/l1`): update the `exec` line and matching `--dry-run` echo. The dispatcher (`verb/<verb>`) rarely changes.
4. If the verb is `test`: also verify `runbox.yml` `test_script` still points to `verb/test` (it should — path does not change when command changes).
5. If the verb is `run`, `lint`, or `build`: update `verb/verbs` to reflect the new command string in the table.
6. Run `verb/test` (or `./run/runbox .test`) to verify the module still passes.
