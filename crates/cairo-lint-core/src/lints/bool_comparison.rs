use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BinaryOperator, Expr, ExprBinary};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

pub const BOOL_COMPARISON: &str = "Unnecessary comparison with a boolean value. Use the variable directly.";

pub fn check_bool_comparison(db: &dyn SyntaxGroup, node: ExprBinary, diagnostics: &mut Vec<PluginDiagnostic>) {
    let lhs = node.lhs(db);
    let op = node.op(db);
    let rhs = node.rhs(db);

    let is_comparison_operator = matches!(op, BinaryOperator::EqEq(_) | BinaryOperator::Neq(_));

    fn is_bool_literal(expr: &Expr) -> bool {
        matches!(expr, Expr::True(_) | Expr::False(_))
    }

    if is_comparison_operator && (is_bool_literal(&lhs) || is_bool_literal(&rhs)) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: node.as_syntax_node().stable_ptr(),
            message: BOOL_COMPARISON.to_string(),
            severity: Severity::Warning,
        });
    }
}
