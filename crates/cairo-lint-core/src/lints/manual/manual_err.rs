use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, ExprIf, ExprMatch};
use cairo_lang_syntax::node::TypedStablePtr;

use crate::lints::manual::{check_manual, check_manual_if, ManualLint};

pub const MANUAL_ERR: &str = "Manual match for `err` detected. Consider using `err()` instead";
pub(super) const LINT_NAME: &str = "manual_err";

pub fn check_manual_err(
    db: &dyn SemanticGroup,
    arenas: &Arenas,
    expr_match: &ExprMatch,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    if check_manual(db, expr_match, arenas, ManualLint::ManualErr, LINT_NAME) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_match.stable_ptr.untyped(),
            message: MANUAL_ERR.to_owned(),
            severity: Severity::Warning,
        });
    }
}

pub fn check_manual_if_err(
    db: &dyn SemanticGroup,
    arenas: &Arenas,
    expr_if: &ExprIf,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    if check_manual_if(db, expr_if, arenas, ManualLint::ManualErr, LINT_NAME) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_if.stable_ptr.untyped(),
            message: MANUAL_ERR.to_owned(),
            severity: Severity::Warning,
        });
    }
}
