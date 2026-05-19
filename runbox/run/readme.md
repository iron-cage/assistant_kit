# run/

Universal runner discovery point for `runbox/example/` projects.

| File | Responsibility |
|------|----------------|
| `runbox-run` | Symlink to `../../dev/run/runbox-run` — the universal container runner. |

Projects at `runbox/example/*/run/` use a walk-up wrapper that discovers `runbox-run` here automatically.
