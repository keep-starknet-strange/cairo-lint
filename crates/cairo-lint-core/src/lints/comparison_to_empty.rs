use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BinaryOperator, Expr};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::ids::SyntaxStablePtrId;
use cairo_lang_syntax::node::{TypedSyntaxNode, TypedStablePtr};

pub const COMPARISON_TO_EMPTY: &str = "Consider using `.is_empty()` instead of comparing to an empty array.";

pub fn check_comparison_to_empty(
    db: &dyn SyntaxGroup,
    expr: &Expr,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    if let Expr::Binary(binary_expr) = expr {
        if let BinaryOperator::Eq(_) = binary_expr.op(db) {
            let lhs = binary_expr.lhs(db);
            let rhs = binary_expr.rhs(db);

            if is_empty_array(db, &lhs) || is_empty_array(db, &rhs) {
                diagnostics.push(create_diagnostic(
                    COMPARISON_TO_EMPTY,
                    expr.stable_ptr().untyped(),
                    Severity::Warning,
                ));
            }
        }
    }
}

fn is_empty_array(db: &dyn SyntaxGroup, expr: &Expr) -> bool {
    if let Expr::FunctionCall(func_call) = expr {
        let func_path = func_call.path(db);
        func_path.as_syntax_node().get_text_without_trivia(db) == "ArrayTrait::new"
    } else {
        false
    }
}

fn create_diagnostic(message: &str, stable_ptr: SyntaxStablePtrId, severity: Severity) -> PluginDiagnostic {
    PluginDiagnostic { stable_ptr, message: message.to_string(), severity }
}