// BUG-289 task/claude_profile/bug/289_son_running_false_haiku_touch_infinite_loop.md — resolve_model Auto gate simplified to son_idle only (Fix: BUG-289, BUG-290)

//! Subprocess model and effort resolution for the `apply_touch` pipeline.
//!
//! `resolve_model` maps `imodel::` + quota data → `IsolatedModel`;
//! `resolve_effort` maps the resolved model + `effort::` → optional effort flag;
//! `effort_pre_args` assembles the `--effort` arg slice for subprocess dispatch.

use super::types::{ AccountQuota, SubprocessModel, SubprocessEffort };

// ── Model resolution ──────────────────────────────────────────────────────────

/// Resolve the subprocess model for one account based on `imodel::` and quota data.
///
/// AC-01: `auto` selects Haiku for general keep-alive pings — Haiku conserves Sonnet
///        and Opus quota.
///        Utilization-aware Sonnet gate (Fix BUG-289, BUG-290, TSK-292; extended Fix BUG-301, TSK-311):
///        Selects `claude-sonnet-4-6` when `seven_day_sonnet` present AND either:
///          • `son_idle=true` (`resets_at=None`) — Haiku cannot start the Sonnet window; a single
///            Sonnet touch opens all idle dimensions simultaneously (5h, 7d, Son); OR
///          • `son_available=true` (`(100 - utilization) > 20%`) — remaining Sonnet quota should
///            not expire unused while Haiku pings keep sessions alive.
///        Falls through to Haiku when Sonnet tier absent or utilization ≥ 80% (≤ 20% remaining).
/// AC-02: `sonnet` always maps to `claude-sonnet-4-6`.
/// AC-03: `opus` always maps to `claude-opus-4-6`.
/// AC-04: `keep` passes `IsolatedModel::KeepCurrent` — no `--model` flag injected.
/// AC-13: `haiku` always maps to `claude-haiku-4-5-20251001`.
#[ inline ]
pub( crate ) fn resolve_model( aq : &AccountQuota, imodel : SubprocessModel ) -> claude_runner_core::IsolatedModel
{
  use claude_runner_core::IsolatedModel;
  match imodel
  {
    SubprocessModel::Sonnet => IsolatedModel::Specific( "claude-sonnet-4-6".to_string() ),
    SubprocessModel::Opus   => IsolatedModel::Specific( "claude-opus-4-6".to_string() ),
    SubprocessModel::Keep   => IsolatedModel::KeepCurrent,
    SubprocessModel::Haiku  => IsolatedModel::Specific( "claude-haiku-4-5-20251001".to_string() ),
    SubprocessModel::Auto   =>
    {
      // Fix(BUG-289, BUG-290, TSK-292): son_idle gate — Haiku cannot activate the 7d-Sonnet window.
      // Fix(BUG-301, TSK-311): son_available gate — remaining Sonnet quota must not expire unused.
      // Sonnet selected when: son_idle (resets_at=None) OR son_available (>20% remaining).
      if let Ok( ref data ) = aq.result
      {
        if let Some( ref son ) = data.seven_day_sonnet
        {
          let son_idle      = son.resets_at.is_none();
          let son_available = 100.0 - son.utilization > 20.0;
          if son_idle || son_available
          {
            return IsolatedModel::Specific( "claude-sonnet-4-6".to_string() );
          }
        }
      }
      // AC-01: Haiku for general keep-alive pings — conserves Sonnet and Opus quota.
      IsolatedModel::Specific( "claude-haiku-4-5-20251001".to_string() )
    }
  }
}

// ── Effort resolution ─────────────────────────────────────────────────────────

/// Resolve the `--effort` flag value for a subprocess given the resolved model.
///
/// Returns `None` when no `--effort` flag should be injected.
/// AC-05: `auto` → `low` for any model that supports effort (Sonnet, Opus); `None` for Haiku or `KeepCurrent`.
///         Haiku has no extended thinking; injecting `--effort` would have no effect or API error.
///         `KeepCurrent` → `None` (model unknown at dispatch time).
/// AC-06: `high` always injects `--effort high`.
/// AC-07: `max` always injects `--effort max`.
/// AC-14: `low` always injects `--effort low`.
/// AC-15: `normal` always injects `--effort normal`.
#[ inline ]
pub( crate ) fn resolve_effort( model : &claude_runner_core::IsolatedModel, effort : SubprocessEffort ) -> Option< &'static str >
{
  use claude_runner_core::IsolatedModel;
  match effort
  {
    SubprocessEffort::High   => Some( "high" ),
    SubprocessEffort::Max    => Some( "max" ),
    SubprocessEffort::Low    => Some( "low" ),
    SubprocessEffort::Normal => Some( "normal" ),
    SubprocessEffort::Auto => match model
    {
      IsolatedModel::Specific( m ) if m.as_str() == "claude-haiku-4-5-20251001" => None,
      IsolatedModel::Specific( _ )                                               => Some( "low" ),
      IsolatedModel::KeepCurrent | IsolatedModel::Default                       => None,
    },
  }
}

/// Build the `extra_pre_args` slice to prepend before `["--print", "."]` in a subprocess.
///
/// Returns `["--effort", value]` when effort resolves to `Some`, otherwise an empty vec.
#[ inline ]
pub( crate ) fn effort_pre_args( model : &claude_runner_core::IsolatedModel, effort : SubprocessEffort ) -> Vec< String >
{
  match resolve_effort( model, effort )
  {
    Some( e ) => vec![ "--effort".to_string(), e.to_string() ],
    None      => vec![],
  }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
mod tests
{
  use super::*;
  use crate::usage::test_support::{ mk_aq_with_sonnet_util, mk_aq_no_sonnet_data, mk_aq_err, mk_aq_with_son_idle };

  // ── resolve_model ──────────────────────────────────────────────────────────

  /// FT-01: `imodel::auto` selects sonnet when 5h absent and `son_idle=true`.
  ///
  /// `mk_aq_with_sonnet_util(85.0)` produces `five_hour=None, son_idle=true`.
  /// Under the `son_idle` gate, Sonnet is selected regardless of 5h state.
  /// Verifies the old `five_h_running` constraint is gone. Fix(BUG-290).
  ///
  /// Spec: [`tests/docs/feature/26_subprocess_model_effort.md` FT-01]
  #[ test ]
  fn it_imodel_auto_selects_sonnet_when_5h_absent()
  {
    let aq       = mk_aq_with_sonnet_util( 85.0 );
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-sonnet-4-6",
      "imodel::auto must select sonnet when son_idle=true (5h absent does not block)",
    );
  }

  /// FT-02: `imodel::auto` selects sonnet with high Sonnet util and `son_idle=true`.
  ///
  /// Utilization percentage is irrelevant for model selection.
  ///
  /// Spec: [`tests/docs/feature/26_subprocess_model_effort.md` FT-02]
  #[ test ]
  fn it_imodel_auto_selects_sonnet_when_5h_absent_high_util()
  {
    let aq       = mk_aq_with_sonnet_util( 35.0 );
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-sonnet-4-6",
      "imodel::auto must select sonnet when son_idle=true regardless of utilization",
    );
  }

  /// FT-03: `imodel::auto` selects sonnet at util boundary and `son_idle=true`.
  ///
  /// Former 20% threshold boundary — utilization is irrelevant for model selection.
  ///
  /// Spec: [`tests/docs/feature/26_subprocess_model_effort.md` FT-03]
  #[ test ]
  fn it_imodel_auto_selects_sonnet_when_5h_absent_boundary_util()
  {
    let aq       = mk_aq_with_sonnet_util( 20.0 );
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-sonnet-4-6",
      "imodel::auto must select sonnet when son_idle=true at util boundary",
    );
  }

  /// FT-04: `imodel::auto` with absent quota data → haiku (quota not needed).
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-04]
  #[ test ]
  fn it_imodel_auto_selects_haiku_without_quota_data()
  {
    let aq       = mk_aq_no_sonnet_data();
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::auto with absent quota data must still select haiku",
    );
  }

  /// EC-9a: `imodel::auto` with account error result → haiku (quota data irrelevant).
  #[ test ]
  fn it_imodel_auto_selects_haiku_with_err_result()
  {
    let aq       = mk_aq_err();
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::auto with Err result must select haiku (quota data not needed)",
    );
  }

  /// Auto model + `effort::auto` → `None` (haiku does not support extended thinking).
  ///
  /// End-to-end: `resolve_model(aq, Auto)` returns haiku; `resolve_effort(haiku, Auto)` → `None`.
  #[ test ]
  fn it_effort_auto_resolves_none_for_auto_haiku()
  {
    let aq     = mk_aq_no_sonnet_data();
    let model  = resolve_model( &aq, SubprocessModel::Auto );
    let effort = resolve_effort( &model, SubprocessEffort::Auto );
    assert!(
      effort.is_none(),
      "imodel::auto + effort::auto must produce None effort (haiku has no extended thinking), got: {effort:?}",
    );
  }

  /// FT-22: `imodel::auto` selects sonnet when `son_idle=true` (5h running, 7d absent).
  ///
  /// The 7d-Sonnet window only activates on Sonnet-family API calls.
  /// Haiku cannot start it. Fix(BUG-289, BUG-290, TSK-292): `son_idle` gate fires.
  ///
  /// Spec: [`tests/docs/feature/26_subprocess_model_effort.md` FT-22]
  #[ test ]
  fn it_imodel_auto_selects_sonnet_when_son_idle()
  {
    let aq       = mk_aq_with_son_idle();
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-sonnet-4-6",
      "imodel::auto must select sonnet when son_idle=true (BUG-289/BUG-290 fix)",
    );
  }

  /// EC-9b: `imodel::auto` selects Haiku when Sonnet window running AND exhausted (≤ 20% remaining).
  ///
  /// `seven_day_sonnet.resets_at=Some(...)` → `son_idle=false`.
  /// `seven_day_sonnet.utilization=90.0` → `son_available=(100-90>20)=false`.
  /// Both gate conditions false → Haiku selected. Conserves the last 10% of Sonnet quota.
  /// Fix(BUG-301, TSK-311): Haiku now requires BOTH gates to fail, not just `son_idle=false`.
  ///
  /// Test Matrix row 2: `five_h=running, d7=running, son=exhausted (90%)` → Haiku.
  #[ test ]
  fn it_imodel_auto_selects_haiku_when_son_running()
  {
    // Start from son_idle base; override son to running and exhausted (90% used, 10% remaining < 20%).
    let mut aq = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq.result
    {
      if let Some( ref mut son ) = data.seven_day_sonnet
      {
        son.resets_at   = Some( "2026-06-14T10:00:00Z".to_string() );
        son.utilization = 90.0;   // Fix(BUG-301): exhausted → Haiku; available (e.g. 60%) → Sonnet
      }
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::auto must select haiku when son_idle=false AND son_available=false (90% used, 10% < 20% threshold)",
    );
  }

  /// EC-9c: `imodel::auto` selects Sonnet when 5h idle + `son_idle=true`.
  ///
  /// `five_hour.resets_at=None` (5h timer present but inactive). `son_idle=true` → gate fires
  /// regardless of 5h state. Verifies old `five_h_running` constraint removed. Fix(BUG-290).
  ///
  /// Test Matrix row 4: `five_h=idle (Some({resets_at:None})), d7=running, son_idle=true` → Sonnet.
  #[ test ]
  fn it_imodel_auto_selects_sonnet_when_5h_idle()
  {
    // Start from son_idle base; override 5h to idle (resets_at=None → five_h_running=false).
    let mut aq = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq.result
    {
      if let Some( ref mut five_h ) = data.five_hour
      {
        five_h.resets_at = None;
      }
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-sonnet-4-6",
      "imodel::auto must select sonnet when son_idle=true (5h idle does not block new gate)",
    );
  }

  /// FT-23: `imodel::auto` selects Haiku when Sonnet tier absent (`seven_day_sonnet=None`).
  ///
  /// `seven_day_sonnet=None` → `son_idle = None.is_some_and(...) = false`.
  /// `son_idle` gate requires `son_idle=true`; absent tier → gate does NOT fire.
  /// `auto` returns Haiku — no Sonnet window exists to start.
  ///
  /// Test Matrix row 3: `five_h=running, d7=None (running), son=absent` → Haiku.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-23]
  #[ test ]
  fn it_imodel_auto_selects_haiku_when_son_tier_absent()
  {
    // Start from son_idle base; remove Sonnet tier entirely → son_idle=false.
    let mut aq = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq.result
    {
      data.seven_day_sonnet = None;
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::auto must select haiku when seven_day_sonnet=None (son_idle=false); gate requires son_idle=true",
    );
  }

  /// FT-24: `imodel::auto` selects Sonnet when 7d timer idle and `son_idle=true`.
  ///
  /// `seven_day=Some({resets_at:None})` → `d7_running=false`.
  /// `son_idle=true` → gate fires regardless of `d7_running` state.
  /// Verifies old `d7_running` constraint removed. Fix(BUG-290).
  ///
  /// Test Matrix row 6: `five_h=running, d7=Some({resets_at:None}) (idle), son_idle=true` → Sonnet.
  ///
  /// Spec: [`tests/docs/feature/26_subprocess_model_effort.md` FT-24]
  #[ test ]
  fn it_imodel_auto_selects_sonnet_when_d7_idle()
  {
    // Start from son_idle base (seven_day=None → d7=true); override to Some(resets_at=None).
    let mut aq = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq.result
    {
      data.seven_day = Some( claude_quota::PeriodUsage { utilization: 50.0, resets_at: None } );
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-sonnet-4-6",
      "imodel::auto must select sonnet when son_idle=true (d7 idle does not block new gate)",
    );
  }

  /// FT-25: `imodel::auto` selects Sonnet when 7d running via explicit Some path and `son_idle=true`.
  ///
  /// `seven_day=Some({resets_at:Some(...)})` exercises `map_or` Some-branch.
  /// `son_idle=true` → gate fires.
  ///
  /// Verifies the `seven_day=Some(running)` path correctly resolves to Sonnet.
  ///
  /// Test Matrix row 7: `five_h=running, d7=Some({resets_at:Some(...)}) (running via Some), son_idle=true` → Sonnet.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-25]
  #[ test ]
  fn it_imodel_auto_selects_sonnet_when_d7_running_explicit()
  {
    // Start from son_idle base; override seven_day to Some(running) — exercises map_or Some branch.
    let mut aq = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq.result
    {
      data.seven_day = Some( claude_quota::PeriodUsage
      {
        utilization : 10.0,
        resets_at   : Some( "2026-06-15T10:00:00Z".to_string() ),
      } );
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-sonnet-4-6",
      "imodel::auto must select sonnet when son_idle=true; d7 and 5h state irrelevant to new gate",
    );
  }

  /// FT-26: `imodel::auto` selects Sonnet when 5h absent + 7d running and `son_idle=true`.
  ///
  /// `five_hour=None` → `five_h_running=false`; `son_idle=true` → gate fires regardless.
  /// This is the BUG-290 cold account scenario. Verifies old `five_h_running` short-circuit
  /// is removed: a single Sonnet touch opens 5h and Son simultaneously. Fix(BUG-290).
  ///
  /// Test Matrix extra row: `five_h=absent, d7=Some(running via Some), son=idle` → Sonnet.
  ///
  /// Spec: [`tests/docs/feature/26_subprocess_model_effort.md` FT-26]
  #[ test ]
  fn it_imodel_auto_selects_sonnet_when_5h_absent_d7_some_running()
  {
    // Start from son_idle base; remove 5h + set d7 to Some(running) to exercise map_or Some-branch.
    let mut aq = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq.result
    {
      data.five_hour  = None;
      data.seven_day  = Some( claude_quota::PeriodUsage
      {
        utilization : 20.0,
        resets_at   : Some( "2026-06-15T10:00:00Z".to_string() ),
      } );
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-sonnet-4-6",
      "imodel::auto must select sonnet when son_idle=true (5h absent does not block new gate)",
    );
  }

  /// FT-27: `imodel::auto` selects Haiku when Sonnet running AND exhausted, regardless of 7d state.
  ///
  /// `seven_day_sonnet.resets_at=Some(...)` → `son_idle=false`.
  /// `seven_day_sonnet.utilization=90.0` → `son_available=(100-90>20)=false`.
  /// Both gate conditions false → Haiku selected. `d7_running` state is irrelevant.
  /// Exercises `son=exhausted` with 7d-idle. Fix(BUG-301, TSK-311).
  ///
  /// Test Matrix extra row: `five_h=running, d7=Some(idle), son=exhausted (90%)` → Haiku.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-27]
  #[ test ]
  fn it_imodel_auto_selects_haiku_when_d7_idle_and_son_running()
  {
    // Start from son_idle base; set d7=Some(idle) and son=exhausted (90% used, 10% remaining < 20%).
    let mut aq = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq.result
    {
      data.seven_day = Some( claude_quota::PeriodUsage { utilization: 50.0, resets_at: None } );
      if let Some( ref mut son ) = data.seven_day_sonnet
      {
        son.resets_at   = Some( "2026-06-14T10:00:00Z".to_string() );
        son.utilization = 90.0;   // Fix(BUG-301): exhausted → Haiku regardless of 7d state
      }
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::auto must select haiku when son_idle=false AND son_available=false (90% used); d7 state irrelevant",
    );
  }

  /// FT-28: `imodel::auto` selects Haiku when Sonnet tier absent and 7d idle.
  ///
  /// `seven_day_sonnet=None` → `son_idle = None.is_some_and(...) = false`. Gate does NOT fire → Haiku.
  /// `d7_running` state is irrelevant to new gate.
  /// Exercises `son=absent(None)` combined with `d7=Some(idle)` — distinct from FT-24 (`son_idle=true`).
  ///
  /// Test Matrix extra row: `five_h=running, d7=Some(idle), son=absent` → Haiku.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-28]
  #[ test ]
  fn it_imodel_auto_selects_haiku_when_d7_idle_and_son_absent()
  {
    // Start from son_idle base; set d7=Some(idle) and remove son tier entirely.
    let mut aq = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq.result
    {
      data.seven_day        = Some( claude_quota::PeriodUsage { utilization: 50.0, resets_at: None } );
      data.seven_day_sonnet = None;
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::auto must select haiku when son_idle=false (absent); d7 state irrelevant to new gate",
    );
  }

  /// FT-29: `imodel::auto` selects Haiku when 7d running via Some + Sonnet tier absent.
  ///
  /// `seven_day=Some({resets_at:Some(...)})` → `d7_running=true` via `map_or` Some-branch.
  /// `seven_day_sonnet=None` → `son_idle = None.is_some_and(...) = false`.
  /// Gate requires `son_idle=true`; absent Sonnet tier blocks it. Haiku selected.
  ///
  /// Completes the `d7=Some(running)` Some-branch column alongside FT-25 (Sonnet, son=idle)
  /// and FT-30 (Haiku, son=running): three cells exhausting the `d7=Z + son=*` combinations.
  ///
  /// Test Matrix row 25: `five_h=running, d7=Some(running via Some), son=absent` → Haiku.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-29]
  #[ test ]
  fn it_imodel_auto_selects_haiku_when_d7_some_running_and_son_absent()
  {
    // Start from son_idle base; override d7 to Some(running) and remove son tier.
    let mut aq = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq.result
    {
      data.seven_day = Some( claude_quota::PeriodUsage
      {
        utilization : 10.0,
        resets_at   : Some( "2026-06-15T10:00:00Z".to_string() ),
      } );
      data.seven_day_sonnet = None;
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::auto must select haiku when d7_running=true (Some-branch) but son_idle=false (son absent); son blocks gate",
    );
  }

  /// MRE — BUG-301: `imodel::auto` selects Haiku when Sonnet window active with 40% remaining.
  ///
  /// Root cause: `subprocess.rs:41` binary `son_idle` gate checked only `resets_at.is_none()`.
  /// When Sonnet window is running (`resets_at=Some`) but 40% quota remains, `son_idle=false` and
  /// the old gate returned Haiku — wasting the remaining quota as the window timer expires.
  /// Fix: extended gate checks `son_available = (100.0 - utilization) > 20.0` alongside `son_idle`.
  ///
  /// Fix(BUG-301, TSK-311).
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-31]
  #[ test ]
  fn mre_bug301_son_active_with_remaining_quota_selects_sonnet()
  {
    // Sonnet window active (resets_at=Some) with 60% utilization (40% remaining > 20% threshold).
    let mut aq = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq.result
    {
      if let Some( ref mut son ) = data.seven_day_sonnet
      {
        son.resets_at   = Some( "2026-06-20T10:00:00Z".to_string() );
        son.utilization = 60.0;
      }
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-sonnet-4-6",
      "imodel::auto must select sonnet when son_idle=false but son_available=true (40% remaining); Fix(BUG-301)",
    );
  }

  /// FT-30: `imodel::auto` selects Haiku when 7d running via Some + Sonnet exhausted.
  ///
  /// `seven_day=Some({resets_at:Some(...)})` → `d7_running=true` via `map_or` Some-branch.
  /// `seven_day_sonnet.resets_at=Some(...)` → `son_idle=false`.
  /// `seven_day_sonnet.utilization=90.0` → `son_available=(100-90>20)=false`.
  /// Both gate conditions false; Haiku selected (Sonnet exhausted, conserve last 10%).
  ///
  /// Completes the `d7=Some(running)` Some-branch column alongside FT-25 (Sonnet, son=idle)
  /// and FT-29 (Haiku, son=absent): all three `son` states under `d7=Z + five_h=running` now covered.
  ///
  /// Test Matrix row 27: `five_h=running, d7=Some(running via Some), son=exhausted` → Haiku.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-30]
  #[ test ]
  fn it_imodel_auto_selects_haiku_when_d7_some_running_and_son_running()
  {
    // Start from son_idle base; override d7 to Some(running) and set son to exhausted (90% used).
    // son_idle=false (resets_at=Some) AND son_available=(100-90>20)=false → both gates fail → Haiku.
    let mut aq = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq.result
    {
      data.seven_day = Some( claude_quota::PeriodUsage
      {
        utilization : 10.0,
        resets_at   : Some( "2026-06-15T10:00:00Z".to_string() ),
      } );
      if let Some( ref mut son ) = data.seven_day_sonnet
      {
        son.resets_at   = Some( "2026-06-14T10:00:00Z".to_string() );
        son.utilization = 90.0;   // Fix(BUG-301): test Haiku via exhaustion, not binary gate
      }
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::auto must select haiku when son_idle=false and son_available=false (90% used, 10% remaining < 20%)",
    );
  }

  /// EC-6: `imodel::sonnet` always returns `IsolatedModel::Specific("claude-sonnet-4-6")`.
  ///
  /// Spec: [`tests/docs/cli/param/035_imodel.md` EC-6]
  #[ test ]
  fn it_imodel_sonnet_explicit()
  {
    let aq       = mk_aq_no_sonnet_data();
    let model    = resolve_model( &aq, SubprocessModel::Sonnet );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!( model_id, "claude-sonnet-4-6", "imodel::sonnet must always return claude-sonnet-4-6" );
  }

  /// EC-7: `imodel::opus` always returns `IsolatedModel::Specific("claude-opus-4-6")`.
  ///
  /// Spec: [`tests/docs/cli/param/035_imodel.md` EC-7]
  #[ test ]
  fn it_imodel_opus_explicit()
  {
    let aq       = mk_aq_no_sonnet_data();
    let model    = resolve_model( &aq, SubprocessModel::Opus );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!( model_id, "claude-opus-4-6", "imodel::opus must always return claude-opus-4-6" );
  }

  /// EC-8: `imodel::keep` returns `IsolatedModel::KeepCurrent` — no `--model` flag.
  ///
  /// Spec: [`tests/docs/cli/param/035_imodel.md` EC-8]
  #[ test ]
  fn it_imodel_keep_no_model_flag()
  {
    let aq    = mk_aq_no_sonnet_data();
    let model = resolve_model( &aq, SubprocessModel::Keep );
    assert!(
      matches!( model, claude_runner_core::IsolatedModel::KeepCurrent ),
      "imodel::keep must return KeepCurrent (no --model flag)",
    );
  }

  // ── Algorithm 001 AC cases ────────────────────────────────────────────────

  /// AC-1 (algorithm/001): `SubprocessModel::Haiku` selects Haiku regardless of quota.
  ///
  /// Even when Sonnet is fully available (`utilization=0.0`, `son_idle=true`),
  /// an explicit `Haiku` param bypasses all quota logic. The explicit-model branch
  /// returns before the Auto quota inspection code runs.
  ///
  /// Spec: [`tests/docs/algorithm/001_touch_model_selection.md` AC-1]
  #[ test ]
  fn ac1_haiku_explicit_selects_haiku_regardless_of_quota()
  {
    // Sonnet fully available and idle — should make no difference for explicit Haiku.
    let aq       = mk_aq_with_sonnet_util( 0.0 );
    let model    = resolve_model( &aq, SubprocessModel::Haiku );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "AC-1: SubprocessModel::Haiku must return claude-haiku-4-5-20251001 regardless of quota; \
       Sonnet availability is irrelevant for explicit imodel",
    );
  }

  /// AC-2 (algorithm/001): `SubprocessModel::Sonnet` selects Sonnet regardless of quota.
  ///
  /// Even when `seven_day_sonnet=None` (no Sonnet tier), an explicit `Sonnet` param
  /// bypasses quota logic. The result is `Specific("claude-sonnet-4-6")` unconditionally.
  ///
  /// Spec: [`tests/docs/algorithm/001_touch_model_selection.md` AC-2]
  #[ test ]
  fn ac2_sonnet_explicit_selects_sonnet_regardless_of_quota()
  {
    // No Sonnet tier — should make no difference for explicit Sonnet.
    let aq       = mk_aq_no_sonnet_data();
    let model    = resolve_model( &aq, SubprocessModel::Sonnet );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-sonnet-4-6",
      "AC-2: SubprocessModel::Sonnet must return claude-sonnet-4-6 regardless of quota; \
       tier absence is irrelevant for explicit imodel",
    );
  }

  /// AC-3 (algorithm/001): `SubprocessModel::Auto` with no Sonnet tier selects Haiku.
  ///
  /// `seven_day_sonnet=None` → `son_idle` and `son_available` gates both unevaluated;
  /// the `if let Some(ref son)` arm does not fire → falls through to Haiku default.
  ///
  /// Spec: [`tests/docs/algorithm/001_touch_model_selection.md` AC-3]
  #[ test ]
  fn ac3_auto_no_sonnet_tier_selects_haiku()
  {
    let aq       = mk_aq_no_sonnet_data();   // seven_day_sonnet = None
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "AC-3: Auto with seven_day_sonnet=None must select Haiku; \
       Sonnet tier absent means neither son_idle nor son_available can fire",
    );
  }

  /// AC-4 (algorithm/001): `SubprocessModel::Auto` with idle Sonnet window selects Sonnet.
  ///
  /// `seven_day_sonnet.resets_at=None` → `son_idle=true`; the idle-window gate fires,
  /// returning Sonnet. Haiku cannot open the idle Sonnet session window; a single
  /// Sonnet touch opens all idle dimensions simultaneously.
  ///
  /// Spec: [`tests/docs/algorithm/001_touch_model_selection.md` AC-4]
  #[ test ]
  fn ac4_auto_idle_sonnet_window_selects_sonnet()
  {
    // utilization=30.0 (70% remaining), resets_at=None → son_idle=true.
    let aq       = mk_aq_with_sonnet_util( 30.0 );
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-sonnet-4-6",
      "AC-4: Auto with son_idle=true (resets_at=None) must select Sonnet; \
       Haiku cannot open an idle Sonnet window",
    );
  }

  /// AC-5 (algorithm/001): `SubprocessModel::Auto` with active window and 25% remaining selects Sonnet.
  ///
  /// `resets_at=Some(...)` → `son_idle=false`; `utilization=75.0` → `son_available=(100-75)>20=true`.
  /// `son_available` gate fires → Sonnet selected to avoid wasting the expiring window.
  /// Regression: Fix BUG-301 (pre-fix, binary `son_idle` gate ignored utilization).
  ///
  /// Spec: [`tests/docs/algorithm/001_touch_model_selection.md` AC-5]
  #[ test ]
  fn ac5_auto_active_sonnet_with_capacity_selects_sonnet()
  {
    // Start from son_idle base; override to active window (resets_at=Some) with 25% remaining.
    let mut aq = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq.result
    {
      if let Some( ref mut son ) = data.seven_day_sonnet
      {
        son.resets_at   = Some( "2026-06-28T04:00:00+00:00".to_string() );
        son.utilization = 75.0;   // 25% remaining > 20% threshold → son_available=true
      }
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-sonnet-4-6",
      "AC-5: Auto with active Sonnet window and 25% remaining must select Sonnet; \
       son_available=(100-75)>20=true; Fix(BUG-301)",
    );
  }

  /// AC-6 (algorithm/001): `SubprocessModel::Auto` with active window and 15% remaining selects Haiku.
  ///
  /// `resets_at=Some(...)` → `son_idle=false`; `utilization=85.0` → `son_available=(100-85)>20=false`.
  /// Both gates fail → Haiku selected to conserve the last 15% Sonnet reserve for direct sessions.
  /// This is the boundary case: exactly at 15% remaining (the ≤20% threshold).
  ///
  /// Spec: [`tests/docs/algorithm/001_touch_model_selection.md` AC-6]
  #[ test ]
  fn ac6_auto_active_sonnet_nearly_exhausted_selects_haiku()
  {
    // Start from son_idle base; override to active window (resets_at=Some) with 15% remaining.
    let mut aq = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq.result
    {
      if let Some( ref mut son ) = data.seven_day_sonnet
      {
        son.resets_at   = Some( "2026-06-28T04:00:00+00:00".to_string() );
        son.utilization = 85.0;   // 15% remaining — NOT > 20% threshold → son_available=false
      }
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "AC-6: Auto with active Sonnet window and 15% remaining must select Haiku; \
       son_available=(100-85)>20=false; conserve reserves for direct sessions",
    );
  }

  // ── resolve_effort ────────────────────────────────────────────────────────

  /// `effort::high` always returns `Some("high")` regardless of model.
  ///
  /// Spec: [`tests/docs/cli/param/036_effort.md` EC-5]
  #[ test ]
  fn it_effort_high_explicit()
  {
    let sonnet = claude_runner_core::IsolatedModel::Specific( "claude-sonnet-4-6".to_string() );
    let opus   = claude_runner_core::IsolatedModel::Specific( "claude-opus-4-6".to_string() );
    let keep   = claude_runner_core::IsolatedModel::KeepCurrent;
    assert_eq!( resolve_effort( &sonnet, SubprocessEffort::High ), Some( "high" ) );
    assert_eq!( resolve_effort( &opus,   SubprocessEffort::High ), Some( "high" ) );
    assert_eq!( resolve_effort( &keep,   SubprocessEffort::High ), Some( "high" ) );
  }

  /// `effort::max` always returns `Some("max")` regardless of model.
  ///
  /// Spec: [`tests/docs/cli/param/036_effort.md` EC-6]
  #[ test ]
  fn it_effort_max_explicit()
  {
    let sonnet = claude_runner_core::IsolatedModel::Specific( "claude-sonnet-4-6".to_string() );
    let opus   = claude_runner_core::IsolatedModel::Specific( "claude-opus-4-6".to_string() );
    let keep   = claude_runner_core::IsolatedModel::KeepCurrent;
    assert_eq!( resolve_effort( &sonnet, SubprocessEffort::Max ), Some( "max" ) );
    assert_eq!( resolve_effort( &opus,   SubprocessEffort::Max ), Some( "max" ) );
    assert_eq!( resolve_effort( &keep,   SubprocessEffort::Max ), Some( "max" ) );
  }

  /// AC-05: `effort::auto` → `low` for Sonnet and Opus; `None` for `KeepCurrent`.
  ///
  /// Spec: [`tests/docs/cli/param/036_effort.md` EC-7–EC-9]
  #[ test ]
  fn it_effort_auto_uniform_low()
  {
    let sonnet = claude_runner_core::IsolatedModel::Specific( "claude-sonnet-4-6".to_string() );
    let opus   = claude_runner_core::IsolatedModel::Specific( "claude-opus-4-6".to_string() );
    let keep   = claude_runner_core::IsolatedModel::KeepCurrent;
    assert_eq!( resolve_effort( &sonnet, SubprocessEffort::Auto ), Some( "low" ), "auto+sonnet must be low" );
    assert_eq!( resolve_effort( &opus,   SubprocessEffort::Auto ), Some( "low" ), "auto+opus must be low" );
    assert_eq!( resolve_effort( &keep,   SubprocessEffort::Auto ), None,          "auto+keep must be None" );
  }

  /// `imodel::keep` + `effort::auto` → no `--effort` flag (`effort_pre_args` returns empty vec).
  ///
  /// Spec: [`tests/docs/cli/param/035_imodel.md` EC-8 interaction note]
  #[ test ]
  fn it_imodel_keep_effort_auto_no_effort_flag()
  {
    let aq       = mk_aq_no_sonnet_data();
    let model    = resolve_model( &aq, SubprocessModel::Keep );
    let pre_args = effort_pre_args( &model, SubprocessEffort::Auto );
    assert!(
      pre_args.is_empty(),
      "imodel::keep + effort::auto must produce no pre-args (no --effort flag), got: {pre_args:?}",
    );
  }

  // ── TSK-209: haiku model + low/normal effort ─────────────────────────────

  /// FT-18 / EC-12 (035): `imodel::haiku` → `IsolatedModel::Specific("claude-haiku-4-5-20251001")`.
  ///
  /// Haiku is explicit-only; `imodel::auto` continues to select between Sonnet and Opus only.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-18]
  ///       [`tests/docs/cli/param/035_imodel.md` EC-12]
  #[ test ]
  fn it_imodel_haiku_explicit()
  {
    let aq       = mk_aq_no_sonnet_data();
    let model    = resolve_model( &aq, SubprocessModel::Haiku );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::haiku must always return claude-haiku-4-5-20251001",
    );
  }

  /// FT-20 / EC-12 (036): `effort::low` always returns `Some("low")` regardless of model.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-20]
  ///       [`tests/docs/cli/param/036_effort.md` EC-12]
  #[ test ]
  fn it_effort_low_explicit()
  {
    let sonnet = claude_runner_core::IsolatedModel::Specific( "claude-sonnet-4-6".to_string() );
    let haiku  = claude_runner_core::IsolatedModel::Specific( "claude-haiku-4-5-20251001".to_string() );
    let keep   = claude_runner_core::IsolatedModel::KeepCurrent;
    assert_eq!( resolve_effort( &sonnet, SubprocessEffort::Low ), Some( "low" ), "effort::low with sonnet must be low" );
    assert_eq!( resolve_effort( &haiku,  SubprocessEffort::Low ), Some( "low" ), "effort::low with haiku must be low" );
    assert_eq!( resolve_effort( &keep,   SubprocessEffort::Low ), Some( "low" ), "effort::low with keep must be low" );
  }

  /// FT-21 / EC-13 (036): `effort::normal` always returns `Some("normal")` regardless of model.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-21]
  ///       [`tests/docs/cli/param/036_effort.md` EC-13]
  #[ test ]
  fn it_effort_normal_explicit()
  {
    let sonnet = claude_runner_core::IsolatedModel::Specific( "claude-sonnet-4-6".to_string() );
    let haiku  = claude_runner_core::IsolatedModel::Specific( "claude-haiku-4-5-20251001".to_string() );
    let keep   = claude_runner_core::IsolatedModel::KeepCurrent;
    assert_eq!( resolve_effort( &sonnet, SubprocessEffort::Normal ), Some( "normal" ), "effort::normal with sonnet must be normal" );
    assert_eq!( resolve_effort( &haiku,  SubprocessEffort::Normal ), Some( "normal" ), "effort::normal with haiku must be normal" );
    assert_eq!( resolve_effort( &keep,   SubprocessEffort::Normal ), Some( "normal" ), "effort::normal with keep must be normal" );
  }

  /// FT-19 / EC-14 (036): `imodel::haiku` + `effort::auto` → `None` (Haiku lacks extended thinking).
  ///
  /// Injecting `--effort` with Haiku would either have no effect or be rejected by the API.
  /// Haiku is the only concrete model that maps to `None` under `effort::auto`.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-19]
  ///       [`tests/docs/cli/param/036_effort.md` EC-14]
  #[ test ]
  fn it_imodel_haiku_effort_auto_no_effort_flag()
  {
    let haiku    = claude_runner_core::IsolatedModel::Specific( "claude-haiku-4-5-20251001".to_string() );
    let pre_args = effort_pre_args( &haiku, SubprocessEffort::Auto );
    assert!(
      pre_args.is_empty(),
      "imodel::haiku + effort::auto must produce no pre-args (no --effort flag), got: {pre_args:?}",
    );
  }
}
