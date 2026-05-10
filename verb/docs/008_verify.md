# Verb: `verify`

- **Kind:** canonical
- **Availability:** universal
- **`--dry-run`:** `w3 .test level::4`

### Command

```bash
w3 .test level::4
```

Level 4 runs: nextest (all features, warnings-as-errors) + doc tests + clippy (-D warnings) + `cargo +nightly udeps` (unused dependency detection) + `cargo +nightly audit` (security vulnerability scan of `Cargo.lock`).

### Notes

`verify` is a superset of `test`. Where `test` gives fast per-PR feedback, `verify` is the full pre-push gate — everything `test` does plus dependency hygiene and security analysis.

`udeps` catches dependencies declared in `Cargo.toml` that are never actually used. `audit` cross-references `Cargo.lock` against the RustSec advisory database. Both require nightly.

`verify` is **identical across all modules** — the command does not vary by module name because `w3 .test level::4` scopes itself from the current workspace context.

Library crates skip the audit step automatically when no `Cargo.lock` is present. `w3` handles this: `[ -f Cargo.lock ] && cargo +nightly audit || echo "ℹ Library crate — skipping audit"`.

`--dry-run` prints `w3 .test level::4` and exits 0 — no checks run.

### Example

```bash
# Any module (command is identical)
./verb/verify              # runs: w3 .test level::4
./verb/verify --dry-run    # prints: w3 .test level::4
```

Relation to `test`:
```
test   → w3 .test level::3   (nextest + doc tests + clippy)
verify → w3 .test level::4   (level::3 + udeps + audit)
```
