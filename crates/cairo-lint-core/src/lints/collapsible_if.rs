use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{Expr, ExprIf, OptionElseClause, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const COLLAPSIBLE_IF: &str =
    "Each `if`-statement adds one level of nesting, which makes code look more complex than it really is.";

pub fn check_collapsible_if(db: &dyn SyntaxGroup, expr_if: &ExprIf, diagnostics: &mut Vec<PluginDiagnostic>) {
    let if_block = expr_if.if_block(db);

    // Verificar si el bloque del if externo contiene solo el if interno
    let statements = if_block.statements(db).elements(db);
    if statements.len() != 1 {
        // Si hay más de una sentencia, no sugerir la combinación
        return;
    }

    if let Some(Statement::Expr(inner_expr_stmt)) = statements.first() {
        if let Expr::If(inner_if_expr) = inner_expr_stmt.expr(db) {
            // Verificar si el if interno tiene un bloque else
            match inner_if_expr.else_clause(db) {
                OptionElseClause::Empty(_) => {
                    // No hay bloque else, podemos continuar
                }
                OptionElseClause::ElseClause(_) => {
                    // Hay un bloque else, no debemos combinar
                    return;
                }
            }

            // Verificar si el if externo tiene un bloque else
            match expr_if.else_clause(db) {
                OptionElseClause::Empty(_) => {
                    // No hay bloque else, podemos continuar
                }
                OptionElseClause::ElseClause(_) => {
                    // Hay un bloque else, no debemos combinar
                    return;
                }
            }

            // Si no hay else y el if interno es la única sentencia, sugerir la combinación
            diagnostics.push(PluginDiagnostic {
                stable_ptr: expr_if.stable_ptr().untyped(),
                message: COLLAPSIBLE_IF.to_string(),
                severity: Severity::Warning,
            });
        }
    }
}
