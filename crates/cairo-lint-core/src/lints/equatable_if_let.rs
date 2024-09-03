use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_syntax::node::ast::{ExprIf, Condition, Expr};
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

pub const EQUATABLE_IF_LET: &str = "`if let` pattern used for equatable value. Consider using a simple comparison `==` instead";

pub fn check_equatable_if_let(
    db: &dyn SyntaxGroup,
    expr: &ExprIf,
    diagnostics: &mut Vec<PluginDiagnostic>
) {
    match expr.condition(db) {
        Condition::Let(condition_let) if is_simple_equality_expr(&condition_let.expr(db)) => {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: expr.as_syntax_node().stable_ptr(),
                message: EQUATABLE_IF_LET.to_string(),
                severity: Severity::Warning,
            });
        }
        _ => {}
    }
}

fn is_simple_equality_expr(expr: &Expr) -> bool {
    match expr {
        // Simple literals like numbers, booleans, and strings
        Expr::Literal(_) | Expr::False(_) | Expr::True(_) | Expr::ShortString(_) | Expr::String(_) => true,

        // Path expression (typically variables or constants)
        Expr::Path(_) => true,

        // If it's any other expression, it’s considered complex
        _ => false,
    }
}


