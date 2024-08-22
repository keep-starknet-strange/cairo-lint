use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{Condition, Expr, ExprLoop, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const LOOP_FOR_WHILE: &str = "unnecessary treatment for loop, use instead while";

pub fn check_loop_for_while(db: &dyn SyntaxGroup, loop_expr: &ExprLoop, diagnostics: &mut Vec<PluginDiagnostic>) {
    let body = loop_expr.body(db);
    let mut has_break = false; // Flag to verify if there is a break

    for statement in body.statements(db).elements(db) {
        // Verify if the statement is an if expression
        if let Statement::Expr(expr_statement) = statement {
            if let Expr::If(if_expr) = expr_statement.expr(db) {
                let condition = if_expr.condition(db);
                
                // Verify if the condition is Let or Expr
                match condition {
                    Condition::Let(_) | Condition::Expr(_) => {
                        // Check for a break statement in the if block
                        let if_block = if_expr.if_block(db);
                        for inner_statement in if_block.statements(db).elements(db) {
                            if let Statement::Break(_) = inner_statement {
                                has_break = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // If a break was found, push the diagnostic
    if has_break {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: loop_expr.stable_ptr().untyped(),
            message: LOOP_FOR_WHILE.to_string(),
            severity: Severity::Warning,
        });
    }
}

