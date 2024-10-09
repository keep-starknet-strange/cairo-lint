use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::Expr;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const DOUBLE_PARENS: &str = "unnecessary double parentheses found. Consider removing them.";

pub const ALLOWED: [&str; 1] = [LINT_NAME];
const LINT_NAME: &str = "double_parens";

pub fn check_double_parens(db: &dyn SyntaxGroup, expr: &Expr, diagnostics: &mut Vec<PluginDiagnostic>) {
    let mut current_node = expr.as_syntax_node();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db, "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }
    let is_double_parens = if let Expr::Parenthesized(parenthesized_expr) = expr {
        matches!(parenthesized_expr.expr(db), Expr::Parenthesized(_) | Expr::Tuple(_))
    } else {
        false
    };

    if is_double_parens {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr.stable_ptr().untyped(),
            message: DOUBLE_PARENS.to_string(),
            severity: Severity::Warning,
        });
    }
}
