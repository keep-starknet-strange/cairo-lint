use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, ExprIf, ExprMatch};
use cairo_lang_syntax::node::TypedStablePtr;

use crate::lints::manual::{check_manual, check_manual_if, ManualLint};

pub const MANUAL_OK_OR: &str = "Manual match for Option<T> detected. Consider using ok_or instead";
pub(super) const LINT_NAME: &str = "manual_ok_or";

pub fn check_manual_ok_or(
    db: &dyn SemanticGroup,
    arenas: &Arenas,
    expr_match: &ExprMatch,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    if check_manual(db, expr_match, arenas, ManualLint::ManualOkOr, LINT_NAME) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_match.stable_ptr.untyped(),
            message: MANUAL_OK_OR.to_owned(),
            severity: Severity::Warning,
        });
    }
}

pub fn check_manual_if_ok_or(
    db: &dyn SemanticGroup,
    arenas: &Arenas,
    expr_if: &ExprIf,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    if check_manual_if(db, expr_if, arenas, ManualLint::ManualOkOr, LINT_NAME) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_if.stable_ptr.untyped(),
            message: MANUAL_OK_OR.to_owned(),
            severity: Severity::Warning,
        });
    }
}
