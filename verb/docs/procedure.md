# Verb Directory Operations

- **Actor:** Developer
- **Trigger:** A new module is added to the workspace and needs a `verb/` directory, OR an existing verb command needs updating across a module.
- **Emits:** —

## Add verb/ Directory to a New Module

1. Determine module type: **binary** (has a `[[bin]]` entry in `Cargo.toml`) or **library** (lib-only).
2. Create `module/<name>/verb/` directory.
3. Create `verb/build`: `cargo build -p <name>` (universal).
4. Create `verb/test` dispatcher (default→l0) + `verb/test.d/l0` (host-native, default) + `verb/test.d/l1` (container-internal, `VERB_LAYER=l1`). All three are universal — identical across all cargo modules.
5. Create `verb/clean`: `cargo clean -p <name>` (universal).
6. Create `verb/run`:
   - **Binary module:** dispatcher (default→l1) + `verb/run.d/l1` (direct: `cargo run -p <name> --bin <binary>`).
   - **Library module:** `echo "verb 'run' is not available for this project" >&2; exit 3` in `verb/run` (no layers needed).
7. Create `verb/lint` dispatcher (default→l1) + `verb/lint.d/l1` (`cargo clippy -p <name> --all-features -- -D warnings`). l1 is module-specific; the dispatcher is universal.
8. Create `verb/verify`: `exec w3 .test level::4` (universal — identical across all modules); add `--dry-run` echo printing `w3 .test level::4`.
9. Create `verb/verbs`: `printf '%-13s ...'` table with verb/status/command for all 8 verbs; library modules show `unavailable` for `run`; last row is `package_info built-in  -`.
10. Create `verb/package_info`: Python3 script — reads `name`, `version`, `edition` from `Cargo.toml` (resolves workspace inheritance via `../../Cargo.toml`), static fields for `language`/`package_manager`/`signal`/`confidence`; prints deterministic flat JSON with blank line before `{` and after `}` (universal — identical across all cargo modules).
11. Create `module/<name>/run/`: `runbox` (auto-discovery wrapper) + `runbox.yml` (`test_script`, `lint_script`, and `run_script` for binary modules; all point to `module/<name>/verb/<verb>.d/l1` — container entry point is l1 directly, bypassing the dispatcher).
12. Set executable bit: `chmod +x module/<name>/verb/*` (dispatchers + plain scripts), `chmod +x module/<name>/verb/*.d/*` (layer files), and `chmod +x module/<name>/run/runbox`.
13. Add `| \`verb/\` | Shell scripts for each \`do\` protocol verb. |` row to `module/<name>/readme.md` Responsibility Table.

## Add verb/ Directory to a Standalone Runbox Project

For non-Rust standalone projects (Python, Node.js, etc.) the `verb/` scripts call ecosystem-native tools directly — same independence principle as Rust modules, just different tools.

1. Determine project type: **binary** (has entry point) or **library** (no runnable entry point).
2. Create `verb/build`: call the ecosystem build tool directly (e.g., `pip install -e .` for Python, `npm install` for Node.js). Does not call `run/runbox` — ecosystem tools run natively on the host.
3. Create `verb/test` dispatcher (default→l0) + `verb/test.d/l0` (host-native runner) + `verb/test.d/l1` (ecosystem runner). Set `test_script: verb/test.d/l1` in `runbox.yml`.
4. Create `verb/clean`: remove build artifacts specific to the ecosystem (`.venv/`, `node_modules/`, `target/`).
5. Create `verb/run`:
   - **Binary project:** dispatcher (default→l1) at `verb/run` + `verb/run.d/l1` (ecosystem entry point); set `run_script: verb/run.d/l1` in `runbox.yml`.
   - **Library project:** `echo "verb 'run' is not available for this project" >&2; exit 3` (no layers needed).
6. Create `verb/lint` dispatcher (default→l1) + `verb/lint.d/l1` (ecosystem linter). Set `lint_script: verb/lint.d/l1` in `runbox.yml`.
7. Create `verb/verify`: call ecosystem-native full checks directly (e.g., `ruff check && pytest` for Python, `eslint . && npm test` for Node.js) — equivalent role to Rust's `w3 .test level::4`. Does not call `run/runbox`.
8. Create `verb/verbs`: same `printf` table format as Rust; ecosystem-specific commands in the table.
9. Create `verb/package_info`: reads the ecosystem manifest (`pyproject.toml`, `package.json`, `Cargo.toml`) and emits flat JSON. Match the field set used by Rust projects.
10. Set executable bit: `chmod +x verb/*` (dispatchers + plain scripts) and `chmod +x verb/*.d/*` (layer files).
11. Add `| \`verb/\` | Shell scripts for each \`do\` protocol verb. |` row to project `readme.md`.

## Update a Verb Command

1. Identify the module and verb to change (e.g., `claude_profile/verb/build`).
2. Read the current script to understand what changes.
3. Edit the layer script (`verb/<verb>.d/l1`): update the `exec` line and matching `--dry-run` echo. The dispatcher (`verb/<verb>`) rarely changes.
4. If the verb is `test`: `l0` and `l1` run the same command — update both layer scripts together. Verify `runbox.yml` `test_script` still points to `verb/test.d/l1` (it should — path does not change when the command inside l1 changes).
5. If the verb is `run`, `lint`, or `build`: update `verb/verbs` to reflect the new command string in the table.
6. Run `./verb/test` to verify the module still passes.
