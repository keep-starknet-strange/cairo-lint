use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::Expr;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const DOUBLE_PARENS: &str = "unnecessary double parentheses found. Consider removing them.";

pub fn check_double_parens(db: &dyn SyntaxGroup, expr: &Expr, diagnostics: &mut Vec<PluginDiagnostic>) {
    let span = match expr {
        Expr::Parenthesized(parenthesized_expr) => {
            if let Expr::Parenthesized(_) | Expr::Tuple(_) = parenthesized_expr.expr(db) {
                Some(parenthesized_expr.as_syntax_node().span(db))
            } else {
                None
            }
        }
        _ => None,
    };

    if let Some(_span) = span {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr.stable_ptr().untyped(),
            message: DOUBLE_PARENS.to_string(),
            severity: Severity::Warning,
        });
    }
}
