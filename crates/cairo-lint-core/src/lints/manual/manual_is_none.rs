use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::ExprMatch;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

use crate::lints::manual::manual::{check_manual, ManualLint};

pub const MANUAL_IS_NONE: &str = "Manual match for `is_none` detected. Consider using `is_none()` instead";

pub fn check_manual_is_none(db: &dyn SyntaxGroup, expr_match: &ExprMatch, diagnostics: &mut Vec<PluginDiagnostic>) {
    if check_manual(db, expr_match, ManualLint::ManualIsNone) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_match.as_syntax_node().stable_ptr(),
            message: MANUAL_IS_NONE.to_owned(),
            severity: Severity::Warning,
        });
    }
}
