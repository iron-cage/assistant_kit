# Verb: `install`

- **Kind:** canonical
- **Availability:** module scope only, binary-only (library modules: exit 3; workspace root: no script)
- **`--dry-run`:** `cargo install --path .` (binary modules only)

### Command

**Binary module:**
```bash
cargo install --path . "$@"
```

Installs the module's binaries to `~/.cargo/bin` via the cargo ecosystem's own installer. `"$@"` forwards all additional arguments to `cargo install` (e.g. `--force`, `--locked`).

**Library module:**
```bash
echo "verb 'install' is not available for this project" >&2
exit 3
```

Exit code 3 signals "verb unavailable" per the `do` protocol, the same contract [`run`](004_run.md) uses — tooling must distinguish exit 3 (not applicable) from exit 1 (tool failure) and exit 127 (tool not found).

### Notes

`install` follows the same binary/library availability rule as [`run`](004_run.md), and over the same seven binary-producing modules: `assistant`, `claude_assets`, `claude_journal_viewer`, `claude_profile`, `claude_runner`, `claude_storage`, `claude_version`. The remaining seventeen modules ship an unconditional exit-3 stub so that `verbs` can report `unavailable` and tooling does not treat a missing file as an error.

Unlike `run`, `install` has **no `.d/` layer directory** — there is no `install.d/l1`, and `VERB_LAYER` does not participate. The script is flat in both forms: a single `cargo install` line for binary modules, a single exit-3 line for library modules. `cargo install --path .` installs every binary the crate declares, so no `--bin` selection is needed where `run` requires one.

There is **no workspace-scope `install`** — `verb/install` does not exist at the workspace root. Installing the aggregate binary is done from its own module (`cargo install --path module/assistant`), which is what the root [`readme.md`](../../readme.md) Quick Start shows.

`install` executes on the host — the container test environment is not involved.

`--dry-run` (first argument) is only defined for binary modules: it prints the cargo invocation and exits 0 without installing. Library modules have no `--dry-run` branch (the script exits 3 unconditionally).

### Example

```bash
# claude_runner (binary module — installs clr, c, claude_runner)
cd module/claude_runner
./verb/install                    # runs: cargo install --path .
./verb/install --dry-run          # prints: cargo install --path .
./verb/install --force            # forwards --force to cargo install

# claude_runner_core (library module)
cd module/claude_runner_core
./verb/install                    # exits 3; stderr: verb 'install' is not available for this project
```

`verbs` output comparison:
```
# binary module (claude_runner)
install  available   cargo install --path .

# library module (claude_runner_core)
install  unavailable -
```
