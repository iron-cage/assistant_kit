# Plugin: Dep cache

- **Status:** 🔒 Hardcoded — mechanism embedded in `runbox.dockerfile`
- **Controls:** How external dependencies are pre-compiled before the test stage
- **Mechanism:** cargo-chef 4-stage build hardcoded in dockerfile (chef → planner → cook → test)

### Notes

Always cargo-chef. No way to switch to sccache or a simpler single-stage build without rewriting the dockerfile.

### Example

`docker build -t workspace_test .` runs the four-stage pipeline:

1. **chef**: `cargo install cargo-chef --locked` into `rust:slim` — cached indefinitely; only rebuilds when the dockerfile's chef stage changes
2. **planner**: `COPY . .` then `cargo chef prepare --recipe-path recipe.json` — produces `recipe.json` from Cargo.toml/Cargo.lock only; stable across `.rs` edits
3. **cook**: receives `recipe.json` only, runs `cargo chef cook --recipe-path recipe.json --workspace --tests` — compiles all external crates (`serde`, `tokio`, `anyhow`, …) into `target/debug/deps/*.rlib`; layer only invalidated by Cargo.toml/Cargo.lock changes
4. **test**: `COPY --from=cook /workspace/target /workspace/target` then `COPY . .` — only workspace crates compile here; all external deps already in `target/debug/deps/` are skipped

Editing a single `.rs` file invalidates the test-stage COPY layer only. The cook layer (all external deps) stays cached — matching the incremental speed of a local `cargo build`.
