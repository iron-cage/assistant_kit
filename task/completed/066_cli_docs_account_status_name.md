# TSK-066: Update CLI docs for `.account.status name::` extension

## Goal

Update all CLI documentation artifacts to reflect the implemented `name::` extension
from FR-16 (TSK-065). Document only what was implemented — no speculative content.

## Motivation

Post-implementation CLI documentation must match the actual behavior of the `name::`
parameter across all verbosity levels, including the N/A behavior for non-active accounts
and the optional/required distinction versus `.account.save/switch/delete`.

## In Scope

- `module/claude_profile/docs/cli/commands.md`:
  - `.account.status` section: param count 2→3; add `name::` row; new examples
- `module/claude_profile/docs/cli/params.md`:
  - `name::` row: commands count 3→4; note optional semantics for `.account.status`
- `module/claude_profile/docs/cli/testing/command/account_status.md`:
  - Add IT-11..IT-NN covering the new test matrix scenarios
  - Update Test Case Index and Coverage Summary
- `module/claude_profile/docs/cli/testing/param/name.md`:
  - Add EC-11: `name::` omitted for `.account.status` → defaults to active account

## Out of Scope

- Implementation changes (complete in TSK-065)
- `spec.md` changes (complete in TSK-063)
- `src/persist.rs` doc gap (complete in TSK-064)

## Work Procedure

**Step 1: `docs/cli/commands.md`**

1. In the commands overview table: update `.account.status` params column `2` → `3`
2. In the `.account.status` H3 section:
   - Update `Parameters:` line: add `name::` link before `v::` and `format::` links
   - Update syntax block: add `clp .account.status name::work` example
   - Add `name::` row to the parameter mini-table (`AccountName`, optional, `""`)
   - Update examples block: add examples at `v::0`, `v::1`, `v::2` with `name::personal`
     showing N/A for email/org on non-active accounts

**Step 2: `docs/cli/params.md`**

1. In the parameter overview table: update `name::` Commands column `3` → `4`
2. In the `### Parameter :: 1. name::` section:
   - Add `.account.status` to the Commands list
   - Update the Default bullet to note context-dependent behavior:
     "(required) for `.account.save`, `.account.switch`, `.account.delete`;
     `""` (shows active account) for `.account.status`"

**Step 3: `docs/cli/testing/command/account_status.md`**

1. Add entries IT-11 through IT-NN from the Phase 3 test matrix (plan 002):
   - IT-11: `name::` = active account → same output as no-name path
   - IT-12: `name::` = non-active account → own expiry, N/A email
   - IT-13: `name::` = nonexistent → exit 2, error message
   - IT-14: `name::` = empty → exit 1, error message
   - IT-15: `name:: v::1` active → shows sub, tier, email, org
   - IT-16: `name:: v::1` non-active → shows sub, tier, N/A email/org
   - IT-17: `name:: v::2` non-active → full metadata with N/A email/org
2. Update Test Case Index table (add IT-11..IT-17)
3. Update Coverage Summary counts

**Step 4: `docs/cli/testing/param/name.md`**

1. Add EC-11: `name::` omitted for `.account.status` → defaults to active account, exit 0

## Validation List

Desired answer for every question is YES.

- [ ] Does `commands.md` `.account.status` section show param count 3?
- [ ] Does `commands.md` `.account.status` mini-table have a `name::` row?
- [ ] Does `commands.md` examples show `name::personal` at multiple verbosities?
- [ ] Does `commands.md` note that email/org shows `N/A` for non-active accounts?
- [ ] Does `params.md` `name::` row show Commands count 4?
- [ ] Does `params.md` `name::` section note optional semantics for `.account.status`?
- [ ] Does `params.md` `name::` section still say required for save/switch/delete?
- [ ] Does `testing/command/account_status.md` have ≥4 new IT entries for `name::`?
- [ ] Does `testing/param/name.md` have EC-11 for optional behavior?
- [ ] Are all cross-references between docs valid (no broken anchor links)?

## Validation Procedure

### Measurements

**M1 — commands.md name:: present for account.status**
```bash
grep "name::" module/claude_profile/docs/cli/commands.md | grep -c "account.status\|AccountName"
```
Before: 0. Expected: ≥1.

**M2 — params.md commands count updated**
```bash
grep -A 3 "^\| 1 \|" module/claude_profile/docs/cli/params.md | grep "4"
```
Before: shows 3. Expected: shows 4.

**M3 — testing file has new IT entries**
```bash
grep -c "IT-1[1-9]\|IT-[2-9][0-9]" module/claude_profile/docs/cli/testing/command/account_status.md
```
Before: 0. Expected: ≥4.

### Anti-faking checks

**AF1 — N/A semantics documented in commands.md**
```bash
grep -c "N/A" module/claude_profile/docs/cli/commands.md
```
Expected: ≥1 (email/org unavailability for non-active accounts documented).

**AF2 — Optional semantics for name:: documented in params.md**
```bash
grep -c "optional\|Optional\|active account" module/claude_profile/docs/cli/params.md
```
Expected: ≥1.

## Outcomes

**Completed:** 2026-03-31
**Result:** Done — updated all 4 CLI documentation files to reflect FR-16 implementation; added 10 new IT entries (IT-11..IT-20) to account_status test doc, 3 new EC entries (EC-11..EC-13) to name:: param test doc.
**Files changed:** `module/claude_profile/docs/cli/commands.md`, `module/claude_profile/docs/cli/params.md`, `module/claude_profile/docs/cli/testing/command/account_status.md`, `module/claude_profile/docs/cli/testing/param/name.md`
