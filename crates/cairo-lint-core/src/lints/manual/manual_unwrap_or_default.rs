use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, ExprIf, ExprMatch};
use cairo_lang_syntax::node::TypedStablePtr;

use crate::lints::manual::{check_manual, check_manual_if, ManualLint};

pub const MANUAL_UNWRAP_OR_DEFAULT: &str = "This can be done in one call with `.unwrap_or_default()`";
pub(super) const LINT_NAME: &str = "manual_unwrap_or_default";

pub fn check_manual_unwrap_or_default(
    db: &dyn SemanticGroup,
    arenas: &Arenas,
    expr_match: &ExprMatch,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    if check_manual(db, expr_match, arenas, ManualLint::ManualUnwrapOrDefault, LINT_NAME) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_match.stable_ptr.untyped(),
            message: MANUAL_UNWRAP_OR_DEFAULT.to_owned(),
            severity: Severity::Warning,
        });
    }
}

pub fn check_manual_if_unwrap_or_default(
    db: &dyn SemanticGroup,
    arenas: &Arenas,
    expr_if: &ExprIf,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    if check_manual_if(db, expr_if, arenas, ManualLint::ManualUnwrapOrDefault, LINT_NAME) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_if.stable_ptr.untyped(),
            message: MANUAL_UNWRAP_OR_DEFAULT.to_owned(),
            severity: Severity::Warning,
        });
    }
}
