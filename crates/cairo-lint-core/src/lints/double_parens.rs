use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::Expr;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const DOUBLE_PARENS: &str = "unnecessary double parentheses found. Consider removing them.";

/// Checks wether an expression is surrounded by useless parensthesis. If it is emits a diagnostic.
/// ```ignore
/// fn foo(x: felt252) -> felt252 {
///     ((x * 2))
/// }
///
/// fn main() -> felt252 {
///     ((foo(10)))
/// }
/// ```
/// This can be rewrote to
///
/// ```ignore
/// fn foo(x: felt252) -> felt252 {
///     x * 2
/// }
///
/// fn main() -> felt252 {
///     foo(10)
/// }
/// ```
pub fn check_double_parens(db: &dyn SyntaxGroup, expr: &Expr, diagnostics: &mut Vec<PluginDiagnostic>) {
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
