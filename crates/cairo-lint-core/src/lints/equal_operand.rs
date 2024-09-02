use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{Expr, ExprBinary};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const REDUNDANT_COMPARISON: &str = "redundant comparison found. This expression always evaluates to true or false.";

pub fn check_redundant_comparison(db: &dyn SyntaxGroup, expr: &ExprBinary, diagnostics: &mut Vec<PluginDiagnostic>) {
    let lhs = expr.lhs(db);
    let rhs = expr.rhs(db);

    let is_function_or_method_call = |expr: &Expr| match expr {
        Expr::FunctionCall(_) => true,
        Expr::StructCtorCall(_) => true,
        _ => false,
    };

    if is_function_or_method_call(&lhs) || is_function_or_method_call(&rhs) {
        return;
    }

    if lhs.as_syntax_node().get_text_without_trivia(db) == rhs.as_syntax_node().get_text_without_trivia(db) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr.stable_ptr().untyped(),
            message: REDUNDANT_COMPARISON.to_string(),
            severity: Severity::Warning,
        });
    }
}
