use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{Expr, ExprLoop, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const LOOP_FOR_WHILE: &str = "you seem to be trying to use `loop`. Consider replacing this `loop` with a `while` \
                                  loop for clarity and conciseness";

pub fn check_loop_for_while(db: &dyn SyntaxGroup, loop_expr: &ExprLoop, diagnostics: &mut Vec<PluginDiagnostic>) {
    let body = loop_expr.body(db);

    for statement in body.statements(db).elements(db) {
        if let Statement::Expr(expr_statement) = statement
            && let Expr::If(if_expr) = expr_statement.expr(db)
        {
            let if_block = if_expr.if_block(db);

            if if_block
                .statements(db)
                .elements(db)
                .iter()
                .any(|inner_statement| matches!(inner_statement, Statement::Break(_)))
            {
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: loop_expr.stable_ptr().untyped(),
                    message: LOOP_FOR_WHILE.to_string(),
                    severity: Severity::Warning,
                });
                return;
            }
        }
    }
}
