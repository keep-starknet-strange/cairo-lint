use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BinaryOperator, Expr, ExprBinary};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{Terminal, TypedStablePtr, TypedSyntaxNode};

pub const ERASE_OP: &str = "This operation will always result in zero and can be simplified.";

pub fn check_expr(db: &dyn SyntaxGroup, expr: &ExprBinary) -> Option<PluginDiagnostic> {
    let lhs = expr.lhs(db);
    let rhs = expr.rhs(db);

    if is_zero_literal(db, &lhs) || is_zero_literal(db, &rhs) {
        let op = expr.op(db);
        if matches!(op, BinaryOperator::Mul(_) | BinaryOperator::Div(_) | BinaryOperator::And(_)) {
            return Some(PluginDiagnostic {
                stable_ptr: expr.stable_ptr().untyped(),
                message: ERASE_OP.to_string(),
                severity: Severity::Error,
            });
        }
    }
    None
}

pub fn is_zero_literal(db: &dyn SyntaxGroup, expr: &Expr) -> bool {
    matches!(expr, Expr::Literal(lit) if lit.text(db) == "0")
}
