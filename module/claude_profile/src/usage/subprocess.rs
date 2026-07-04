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
///        Selects `claude-sonnet-5` when `seven_day_sonnet` present AND either:
///          • `son_idle=true` (`resets_at=None`) — Haiku cannot start the Sonnet window; a single
///            Sonnet touch opens all idle dimensions simultaneously (5h, 7d, Son); OR
///          • `son_available=true` (`(100 - utilization) > 20%`) — remaining Sonnet quota should
///            not expire unused while Haiku pings keep sessions alive.
///        Falls through to Haiku when Sonnet tier absent or utilization ≥ 80% (≤ 20% remaining).
/// AC-02: `sonnet` always maps to `claude-sonnet-5`.
/// AC-03: `opus` always maps to `claude-opus-4-8`.
/// AC-04: `keep` passes `IsolatedModel::KeepCurrent` — no `--model` flag injected.
/// AC-13: `haiku` always maps to `claude-haiku-4-5-20251001`.
#[ must_use ]
#[ inline ]
pub fn resolve_model( aq : &AccountQuota, imodel : SubprocessModel ) -> claude_runner_core::IsolatedModel
{
  use claude_runner_core::IsolatedModel;
  match imodel
  {
    SubprocessModel::Sonnet => IsolatedModel::Specific( "claude-sonnet-5".to_string() ),
    SubprocessModel::Opus   => IsolatedModel::Specific( "claude-opus-4-8".to_string() ),
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
            return IsolatedModel::Specific( "claude-sonnet-5".to_string() );
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
#[ must_use ]
#[ inline ]
pub fn resolve_effort( model : &claude_runner_core::IsolatedModel, effort : SubprocessEffort ) -> Option< &'static str >
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
#[ must_use ]
#[ inline ]
pub fn effort_pre_args( model : &claude_runner_core::IsolatedModel, effort : SubprocessEffort ) -> Vec< String >
{
  match resolve_effort( model, effort )
  {
    Some( e ) => vec![ "--effort".to_string(), e.to_string() ],
    None      => vec![],
  }
}


// Tests live in tests/usage/subprocess_tests.rs (integration tests via test_bridge).
