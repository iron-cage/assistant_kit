> **DEPRECATED:** Removed with `--verbosity` parameter. Replace all call sites with `!quiet` bool. Delete `src/verbosity.rs`. See `074_quiet.md` for replacement.

# CLI Type: VerbosityLevel

Removed in v1.2.0 (TSK-337, Plan 038). `src/verbosity.rs` and the `VerbosityLevel` newtype were deleted.

`VerbosityLevel` was a u8 newtype (0–5) controlling runner diagnostic output gating via six predicate methods (`shows_errors()`, `shows_warnings()`, `shows_progress()`, `shows_verbose_detail()`, `shows_debug()`). Replaced by the `--quiet` bool flag.

See [`012_verbosity.md`](../param/012_verbosity.md) for historical parameter context and [`074_quiet.md`](../param/074_quiet.md) for the replacement.
