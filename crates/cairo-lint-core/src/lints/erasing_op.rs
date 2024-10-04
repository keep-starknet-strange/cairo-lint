use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BinaryOperator, Expr, ExprBinary};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode};

pub const ERASING_OPERATION: &str = "This operation results in the value being erased (e.g., multiplication by 0). \
                                     Consider replacing the entire expression with 0.";

pub fn check_erasing_operation(db: &dyn SyntaxGroup, node: ExprBinary, diagnostics: &mut Vec<PluginDiagnostic>) {
    let lhs = node.lhs(db);
    let op = node.op(db);
    let rhs = node.rhs(db);

    let is_erasing_operation = match op {
        BinaryOperator::Mul(_) | BinaryOperator::Div(_) => is_zero(db, &lhs) || is_zero(db, &rhs),
        BinaryOperator::And(_) => is_zero(db, &lhs) || is_zero(db, &rhs),
        _ => false,
    };
    fn is_zero(db: &dyn SyntaxGroup, expr: &Expr) -> bool {
        match expr {
            Expr::Literal(lit) => lit.text(db) == "0",
            _ => false,
        }
    }
    if is_erasing_operation {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: node.as_syntax_node().stable_ptr(),
            message: ERASING_OPERATION.to_string(),
            severity: Severity::Warning,
        });
    }
}
