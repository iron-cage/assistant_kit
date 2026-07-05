# decisions — claude_version

Design decisions for the claude_version crate.

State legend: 🔍 Unverified · ✔️ Verified · 🚧 Blocked · ✅ Decided · ➖ Cancelled

Format rule: ✅ Decided entries show only the selected option as a single statement with rationale. ✔️ Verified entries retain the Options section pending human ENDORSE. 🔍 Unverified entries show the assumed decision with contingency or full option structure. 🚧 Blocked entries retain full option structure. ➖ Cancelled entries preserve original analysis with a mandatory Reason field.

---

## Index

| ID | Question | State | Owner | Date | Gated by |
|----|----------|-------|-------|------|----------|
| Q-01 | What mechanism should trace mutating function calls? | 🔍 Unverified | dev | 2026_07_04 | — |
| Q-02 | Should lock-layer state extend `.status` or a new command? | 🔍 Unverified | dev | 2026_07_04 | — |

---

## Q-01 — Parameter-Trace Instrumentation Mechanism

**🔍 Unverified · dev · 2026_07_04**
What mechanism should trace mutating function calls?

**Assumed unconditional `eprintln!` trace lines** at the entry point of every public mutating function, with no new logging crate dependency, based on the existing ungated `eprintln!` diagnostic idiom already used in the `claude_version` CLI crate at `src/commands/version.rs:263,274,448` and `src/lib.rs:243,263,275,293` (not yet present in `claude_version_core` itself — this task introduces the convention there for the first time) and the absence of any `log`/`tracing` dependency in either `claude_version/Cargo.toml` or `claude_version_core/Cargo.toml`. Validated by Task 006's Verification Gate and code review. If wrong (trace volume becomes unmanageable or structured/filterable output is later needed): introduce a proper logging crate (`log`+`env_logger` or `tracing`) as a follow-up task, and INVALIDATE this entry.

---

## Q-02 — Lock-State Visibility Surface

**🔍 Unverified · dev · 2026_07_04**
Should lock-layer state extend `.status` or a new command?

**Assumed extending the existing `.status` command** (`module/claude_version/src/commands/status.rs`) with a new higher-verbosity "Lock:" section reporting all 5 pattern layers' actual vs. expected state, rather than creating a new dedicated command, based on `.status` already reporting the Layer 5 preferred-version signal and already having a verbosity-tiered output path (`opts.verbosity`) — a new command would duplicate `.status`'s existing "installation state" responsibility (One-Second Test). Validated by Task 007's Verification Gate. If wrong (output becomes too dense or conflates concerns): split into a dedicated `.version.lock_status` command as a follow-up, and INVALIDATE this entry.

---
