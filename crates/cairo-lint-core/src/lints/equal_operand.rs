use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Expr, ExprIf};
use cairo_lang_syntax::node::kind::SyntaxKind;

pub const REDUNDANT_COMPARISON: &str = "redundant comparison found. This expression always evaluates to true or false.";

pub fn check_redundant_comparison(
    db: &dyn SemanticGroup,
    if_expr: &ExprIf,
    diagnostics: &mut Vec<PluginDiagnostic>,
    arenas: &Arenas,
) {
    let lhs = expr.lhs(db);
    let rhs = expr.rhs(db);

    let is_function_or_method_call = |expr: &Expr| {
        let syntax_node = expr.as_syntax_node();
        match syntax_node.kind(db) {
            SyntaxKind::ExprFunctionCall | SyntaxKind::ExprStructCtorCall => true,
            _ => false,
        }
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
