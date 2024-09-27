use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::ExprMatch;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

use crate::lints::manual::{check_manual, ManualLint};

pub const MANUAL_EXPECT: &str = "Manual match for expect detected. Consider using `expect()` instead";

pub fn check_manual_expect(db: &dyn SyntaxGroup, expr_match: &ExprMatch, diagnostics: &mut Vec<PluginDiagnostic>) {
    if check_manual(db, expr_match, ManualLint::ManualExpect) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_match.as_syntax_node().stable_ptr(),
            message: MANUAL_EXPECT.to_owned(),
            severity: Severity::Warning,
        });
    }
}
