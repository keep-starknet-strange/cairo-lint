use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, ExprIf, ExprMatch};
use cairo_lang_syntax::node::TypedStablePtr;

use crate::lints::manual::{check_manual, check_manual_if, ManualLint};

pub const MANUAL_EXPECT: &str = "Manual match for expect detected. Consider using `expect()` instead";

pub(super) const LINT_NAME: &str = "manual_expect";

pub fn check_manual_expect(
    db: &dyn SemanticGroup,
    arenas: &Arenas,
    expr_match: &ExprMatch,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    if check_manual(db, expr_match, arenas, ManualLint::ManualOptExpect, LINT_NAME) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_match.stable_ptr.untyped(),
            message: MANUAL_EXPECT.to_owned(),
            severity: Severity::Warning,
        });
    }

    if check_manual(db, expr_match, arenas, ManualLint::ManualResExpect, LINT_NAME) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_match.stable_ptr.untyped(),
            message: MANUAL_EXPECT.to_owned(),
            severity: Severity::Warning,
        });
    }
}

pub fn check_manual_if_expect(
    db: &dyn SemanticGroup,
    arenas: &Arenas,
    expr_if: &ExprIf,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    if check_manual_if(db, expr_if, arenas, ManualLint::ManualOptExpect, LINT_NAME) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_if.stable_ptr.untyped(),
            message: MANUAL_EXPECT.to_owned(),
            severity: Severity::Warning,
        });
    }

    if check_manual_if(db, expr_if, arenas, ManualLint::ManualResExpect, LINT_NAME) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_if.stable_ptr.untyped(),
            message: MANUAL_EXPECT.to_owned(),
            severity: Severity::Warning,
        });
    }
}
