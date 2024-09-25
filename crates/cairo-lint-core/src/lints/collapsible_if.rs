use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{Expr, ExprIf, OptionElseClause, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const COLLAPSIBLE_IF: &str =
    "Each `if`-statement adds one level of nesting, which makes code look more complex than it really is.";

pub fn check_collapsible_if(db: &dyn SyntaxGroup, expr_if: &ExprIf, diagnostics: &mut Vec<PluginDiagnostic>) {
    let if_block = expr_if.if_block(db);

    let statements = if_block.statements(db).elements(db);
    if statements.len() != 1 {
        return;
    }

    if let Some(Statement::Expr(inner_expr_stmt)) = statements.first() {
        if let Expr::If(inner_if_expr) = inner_expr_stmt.expr(db) {
            match inner_if_expr.else_clause(db) {
                OptionElseClause::Empty(_) => {
                }
                OptionElseClause::ElseClause(_) => {
                    return;
                }
            }

            match expr_if.else_clause(db) {
                OptionElseClause::Empty(_) => {

                }
                OptionElseClause::ElseClause(_) => {

                    return;
                }
            }

            diagnostics.push(PluginDiagnostic {
                stable_ptr: expr_if.stable_ptr().untyped(),
                message: COLLAPSIBLE_IF.to_string(),
                severity: Severity::Warning,
            });
        }
    }
}
