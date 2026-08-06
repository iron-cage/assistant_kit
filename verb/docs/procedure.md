# Verb Directory Operations

- **Actor:** Developer
- **Trigger:** A new module is added to the workspace and needs a `verb/` directory, OR an existing verb command needs updating across a module.
- **Emits:** —

## Add verb/ Directory to a New Module

1. Determine module type: **binary** (has a `[[bin]]` entry in `Cargo.toml`) or **library** (lib-only).
2. Create `module/<name>/verb/` directory.
3. Create `verb/build`: `cargo build -p <name>` (universal).
4. Create `verb/test` (rejects any host `VERB_LAYER`; execs `runbox .live -- ./module/<name>/verb/test.d/l1`) + `verb/test.d/l0` (disabled hard-error stub) + `verb/test.d/l1` (container-internal: cd to the module dir, then nextest + doc tests + clippy). Add `verb/test_only` + `verb/test_only.d/l1` (targeted run, positional filter). All are universal — identical across cargo modules up to the module name.
5. Create `verb/clean`: `cargo clean -p <name>` (universal).
6. Create `verb/run`:
   - **Binary module:** dispatcher (default→l1) + `verb/run.d/l1` (direct: `cargo run -p <name> --bin <binary>`).
   - **Library module:** `echo "verb 'run' is not available for this project" >&2; exit 3` in `verb/run` (no layers needed).
7. Create `verb/lint` dispatcher (default→l1) + `verb/lint.d/l1` (`cargo clippy -p <name> --all-features -- -D warnings`). l1 is module-specific; the dispatcher is universal.
8. Create `verb/verify`: `exec will .test level::4` (universal — identical across all modules); add `--dry-run` echo printing `will .test level::4`.
9. Create `verb/verbs`: `printf '%-13s ...'` table with verb/status/command for all verbs; library modules show `unavailable` for `run`; last row is `package_info built-in  -`.
10. Create `verb/package_info`: Python3 script — reads `name`, `version`, `edition` from `Cargo.toml` (resolves workspace inheritance via `../../Cargo.toml`), static fields for `language`/`package_manager`/`signal`/`confidence`; prints deterministic flat JSON with blank line before `{` and after `}` (universal — identical across all cargo modules).
11. No per-module container config — the workspace's owning `runbox/runbox.yml` serves every module; `verb/test` passes the module layer as the `runbox .live` payload.
12. Set executable bit: `chmod +x module/<name>/verb/*` (dispatchers + plain scripts) and `chmod +x module/<name>/verb/*.d/*` (layer files).
13. Add `| \`verb/\` | Shell scripts for each \`do\` protocol verb. |` row to `module/<name>/readme.md` Responsibility Table.

## Add verb/ Directory to a Standalone Runbox Project

For non-Rust standalone projects (Python, Node.js, etc.) the `verb/` scripts call ecosystem-native tools directly — same independence principle as Rust modules, just different tools.

1. Determine project type: **binary** (has entry point) or **library** (no runnable entry point).
2. Create `verb/build`: call the ecosystem build tool directly (e.g., `pip install -e .` for Python, `npm install` for Node.js). Does not call the `runbox` engine — ecosystem tools run natively on the host.
3. Create `verb/test` dispatcher (default→l0) + `verb/test.d/l0` (host-native runner) + `verb/test.d/l1` (ecosystem runner). Set `script: verb/test.d/l1` in the project's `runbox.yml` — the engine's `.run`/`.live` payload default.
4. Create `verb/clean`: remove build artifacts specific to the ecosystem (`.venv/`, `node_modules/`, `target/`).
5. Create `verb/run`:
   - **Binary project:** dispatcher (default→l1) at `verb/run` + `verb/run.d/l1` (ecosystem entry point). Runs host-side — the container config carries only `script:`.
   - **Library project:** `echo "verb 'run' is not available for this project" >&2; exit 3` (no layers needed).
6. Create `verb/lint` dispatcher (default→l1) + `verb/lint.d/l1` (ecosystem linter). Host-side as well.
7. Create `verb/verify`: call ecosystem-native full checks directly (e.g., `ruff check && pytest` for Python, `eslint . && npm test` for Node.js) — equivalent role to Rust's `will .test level::4`. Does not call the `runbox` engine.
8. Create `verb/verbs`: same `printf` table format as Rust; ecosystem-specific commands in the table.
9. Create `verb/package_info`: reads the ecosystem manifest (`pyproject.toml`, `package.json`, `Cargo.toml`) and emits flat JSON. Match the field set used by Rust projects.
10. Set executable bit: `chmod +x verb/*` (dispatchers + plain scripts) and `chmod +x verb/*.d/*` (layer files).
11. Add `| \`verb/\` | Shell scripts for each \`do\` protocol verb. |` row to project `readme.md`.

## Add Workspace-Level Verbs

Workspace-level verbs operate on all crates simultaneously. They live in the workspace root `verb/` directory alongside the existing `test` dispatcher.

1. Create `verb/build`: `cargo build --workspace` (universal).
2. `verb/test` already exists — execs `runbox .live` (container; the owning `runbox/runbox.yml` names `verb/test.d/l1` as `script:`, the full `--workspace` suite).
3. Create `verb/clean`: `cargo clean` (workspace-wide, no `-p` scoping needed).
4. Create `verb/run`: `echo "error: run is unavailable at workspace scope" >&2; exit 3` — workspace has multiple binaries; use `module/*/verb/run` instead.
5. Create `verb/lint`: `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
6. Create `verb/verify`: chains `./verb/test` (container) + `cargo +nightly udeps --workspace --all-targets --all-features` + `cargo +nightly audit` (host). Not a single `exec` — multi-step.
7. Create `verb/verbs`: `printf` table showing the workspace verbs with `run` as `unavailable`.
8. Create `verb/package_info`: Python3 script reading workspace `Cargo.toml` — emits JSON with `scope: "workspace"`, member count, crate name list, workspace version/edition.
9. Set executable bit: `chmod +x verb/{build,clean,lint,run,verify,verbs,package_info}`.
10. Update `verb/readme.md` Responsibility Table with rows for all new verbs.

All workspace verbs follow the same `--dry-run` contract as module verbs: print the command, exit 0.

## Update a Verb Command

1. Identify the module and verb to change (e.g., `claude_profile/verb/build`).
2. Read the current script to understand what changes.
3. Edit the layer script (`verb/<verb>.d/l1`): update the `exec` line and matching `--dry-run` echo. The dispatcher (`verb/<verb>`) rarely changes.
4. If the verb is `test`: update `verb/test.d/l1` only (`l0` is a disabled stub and carries no command). The owning `runbox/runbox.yml` `script:` names the workspace `verb/test.d/l1`; module payloads are passed explicitly by each module's `verb/test` — neither changes when the command inside l1 changes.
5. If the verb is `run`, `lint`, or `build`: update `verb/verbs` to reflect the new command string in the table.
6. Run `./verb/test` to verify the module still passes.
