> **DEPRECATED:** Removed in favour of `--quiet` ([074_quiet.md](074_quiet.md)). The numeric 0–5 scale bundles independent output concerns into one opaque integer; `--quiet` is the focused boolean replacement. Remove code: delete `src/verbosity.rs`, `VerbosityLevel` newtype, `--verbosity` parse arm; replace gate sites with `!quiet`.

# CLI Parameter: --verbosity (DEPRECATED)

Removed in v1.2.0 (TSK-337, Plan 038). Replaced by `--quiet` ([074_quiet.md](074_quiet.md)).

Historical context: u8 value 0–5 (default 3); type [`VerbosityLevel`](../type/05_verbosity_level.md); group [Runner Control](../param_group/02_runner_control.md); commands: `run`, `ask`. Fatal errors always bypassed the gate (BUG-240). `--dry-run` output was similarly unaffected.
