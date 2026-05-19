# runbox

Universal container test runner. Proves the runbox approach works across ecosystems via working examples.

| Path | Responsibility |
|------|----------------|
| `run/` | `runbox-run` symlink — universal runner discovered by project wrappers via walk-up. |
| `example/` | Working examples: one per ecosystem, each fully integrated with runbox and verb protocol. |

### Verb Protocol

The verb protocol defines the interface between user-facing operations and their execution-layer implementations. Two forms:

- **Flat file** `verb/X` — single behavior; `VERB_LAYER` is ignored. Use for verbs with identical behavior everywhere (`build`, `clean`, `verify`).
- **Dispatcher + layers** `verb/X` (dispatcher file) + `verb/X.d/` (layers directory) — multi-layer; the dispatcher reads `VERB_LAYER` and self-dispatches to the correct layer file (`test.d/l1`, `test.d/l2`). The final `exec` line in the dispatcher encodes the default layer.

`runbox-run` sets `VERB_LAYER=l1` when invoking verbs inside the container. `verb/X` is always a file — never a directory. See `dev/run/onboarding.md § Multi-Layer Verbs` for full protocol specification.
