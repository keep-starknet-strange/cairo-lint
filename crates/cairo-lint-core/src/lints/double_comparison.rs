use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BinaryOperator, Expr, ExprBinary};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const DOUBLE_COMPARISON: &str = "redundant double comparison found. Consider simplifying to a single comparison.";

pub fn check_double_comparison(db: &dyn SyntaxGroup, expr: &Expr, diagnostics: &mut Vec<PluginDiagnostic>) {
    if let Expr::Binary(binary_op) = expr {
        let lhs = binary_op.lhs(db);
        let rhs = binary_op.rhs(db);

        if let (Some(lhs_op), Some(rhs_op)) = (extract_binary_operator(&lhs, db), extract_binary_operator(&rhs, db)) {
            if is_redundant_double_comparison(&lhs_op, &rhs_op) {
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: expr.stable_ptr().untyped(),
                    message: DOUBLE_COMPARISON.to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }
}

fn is_redundant_double_comparison(lhs_op: &BinaryOperator, rhs_op: &BinaryOperator) -> bool {
    matches!(
        (lhs_op, rhs_op),
        (BinaryOperator::EqEq(_), BinaryOperator::LT(_))
            | (BinaryOperator::LT(_), BinaryOperator::EqEq(_))
            | (BinaryOperator::EqEq(_), BinaryOperator::GT(_))
            | (BinaryOperator::GT(_), BinaryOperator::EqEq(_))
            | (BinaryOperator::GE(_), BinaryOperator::GT(_))
            | (BinaryOperator::LE(_), BinaryOperator::LT(_))
            | (BinaryOperator::LE(_), BinaryOperator::GE(_))
            | (BinaryOperator::GE(_), BinaryOperator::LE(_))
            | (BinaryOperator::LT(_), BinaryOperator::GT(_))
            | (BinaryOperator::GT(_), BinaryOperator::LT(_))
    )
}

pub fn extract_binary_operator(expr: &Expr, db: &dyn SyntaxGroup) -> Option<BinaryOperator> {
    if let Expr::Binary(binary_op) = expr { Some(binary_op.op(db)) } else { None }
}

pub fn extract_variable(binary_expr: &ExprBinary, db: &dyn SyntaxGroup) -> String {
    let lhs = binary_expr.lhs(db);
    lhs.as_syntax_node().get_text(db).to_string()
}

pub fn operator_to_replace(lhs_op: BinaryOperator) -> &'static str {
    match lhs_op {
        BinaryOperator::EqEq(_) => "==",
        BinaryOperator::GT(_) => ">",
        BinaryOperator::LT(_) => "<",
        BinaryOperator::GE(_) => ">=",
        BinaryOperator::LE(_) => "<=",
        _ => "",
    }
}
