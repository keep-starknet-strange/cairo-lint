use cairo_lang_syntax::node::ast::{ExprBinary, Expr, BinaryOperator};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;

use cairo_lang_syntax::node::Terminal;
use cairo_lang_syntax::node::{TypedSyntaxNode, TypedStablePtr}; 

#[derive(Default, Debug)]
pub struct EraseOp;

impl EraseOp {
    pub fn new() -> Self {
        Self
    }

    pub fn check_expr(&self, db: &dyn SyntaxGroup, expr: &ExprBinary) -> Option<PluginDiagnostic> {
        let lhs = expr.lhs(db);
        let rhs = expr.rhs(db);

        if self.is_zero_literal(db, &lhs) || self.is_zero_literal(db, &rhs) {
            let op = expr.op(db);
            if matches!(op, BinaryOperator::Mul(_) | BinaryOperator::Div(_)) {
                return Some(PluginDiagnostic {
                    stable_ptr: expr.stable_ptr().untyped(),
                    message: "This operation will always result in zero and can be simplified.".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
        None
    }

    fn is_zero_literal(&self, db: &dyn SyntaxGroup, expr: &Expr) -> bool {
        matches!(expr, Expr::Literal(lit) if lit.text(db) == "0")
    }
}



