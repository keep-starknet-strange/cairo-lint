use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{Expr, ExprIf, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const COLLAPSIBLE_IF: &str =
    "Each `if`-statement adds one level of nesting, which makes code look more complex than it really is.";

pub fn check_collapsible_if(db: &dyn SyntaxGroup, expr_if: &ExprIf, diagnostics: &mut Vec<PluginDiagnostic>) {
    let if_block = expr_if.if_block(db);

    // Combina las condiciones utilizando `if let` y verificando directamente el `inner_if_expr`.
    if let Some(Statement::Expr(expr_stmt)) = if_block.statements(db).elements(db).first()
        && let Expr::If(inner_if_expr) = expr_stmt.expr(db)
        && inner_if_expr.if_block(db).statements(db).elements(db).iter().all(|stmt| matches!(stmt, Statement::Expr(_)))
    {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_if.stable_ptr().untyped(),
            message: COLLAPSIBLE_IF.to_string(),
            severity: Severity::Warning,
        });
    }
}
