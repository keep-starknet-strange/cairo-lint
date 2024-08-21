use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BinaryOperator, Expr};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const DOUBLE_COMPARISON: &str = "redundant double comparison found. Consider simplifying to a single comparison.";

pub fn check_double_comparison(db: &dyn SyntaxGroup, expr: &Expr, diagnostics: &mut Vec<PluginDiagnostic>) {
    if let Expr::Binary(binary_op) = expr {
        let lhs = binary_op.lhs(db);
        let rhs = binary_op.rhs(db);

        if let (Some(lhs_op), Some(rhs_op)) = (extract_binary_operator(&lhs, db), extract_binary_operator(&rhs, db)) {
            if (matches!(lhs_op, BinaryOperator::EqEq(_)) && matches!(rhs_op, BinaryOperator::LT(_)))
                || (matches!(lhs_op, BinaryOperator::LT(_)) && matches!(rhs_op, BinaryOperator::EqEq(_)))
                || (matches!(lhs_op, BinaryOperator::GE(_)) && matches!(rhs_op, BinaryOperator::GT(_)))
                || (matches!(lhs_op, BinaryOperator::LE(_)) && matches!(rhs_op, BinaryOperator::LT(_)))
                || (matches!(lhs_op, BinaryOperator::EqEq(_)) && matches!(rhs_op, BinaryOperator::GT(_)))
                || (matches!(lhs_op, BinaryOperator::GT(_)) && matches!(rhs_op, BinaryOperator::EqEq(_)))
                || (matches!(lhs_op, BinaryOperator::LE(_)) && matches!(rhs_op, BinaryOperator::GE(_)))
                || (matches!(lhs_op, BinaryOperator::GE(_)) && matches!(rhs_op, BinaryOperator::LE(_)))
                || (matches!(lhs_op, BinaryOperator::LT(_)) && matches!(rhs_op, BinaryOperator::GT(_)))
                || (matches!(lhs_op, BinaryOperator::GT(_)) && matches!(rhs_op, BinaryOperator::LT(_)))
            {
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: expr.stable_ptr().untyped(),
                    message: DOUBLE_COMPARISON.to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }
}

fn extract_binary_operator(expr: &Expr, db: &dyn SyntaxGroup) -> Option<BinaryOperator> {
    if let Expr::Binary(binary_op) = expr { Some(binary_op.op(db)) } else { None }
}
