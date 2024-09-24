use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BlockOrIf, Condition, Expr, ExprIf, OptionElseClause};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

pub const DUPLICATE_IF_CONDITION: &str = "Consecutive `if` with the same condition found.";

fn is_function_call(expr: &Expr) -> bool {
    matches!(expr, Expr::FunctionCall(_))
}

fn get_condition_text(db: &dyn SyntaxGroup, condition: &Condition) -> Option<String> {
    match condition {
        Condition::Expr(expr) => Some(expr.as_syntax_node().get_text(db).to_string()),
        Condition::Let(condition_let) => Some(condition_let.expr(db).as_syntax_node().get_text(db).to_string()),
    }
}

pub fn check_duplicate_if_condition(db: &dyn SyntaxGroup, if_expr: &ExprIf, diagnostics: &mut Vec<PluginDiagnostic>) {
    let if_condition = if_expr.condition(db);
    if let OptionElseClause::ElseClause(else_clause) = if_expr.else_clause(db) {
        if let BlockOrIf::If(else_if_block) = else_clause.else_block_or_if(db) {
            let else_if_condition = else_if_block.condition(db);

            let is_if_function_call = match &if_condition {
                Condition::Expr(expr) => is_function_call(&expr.expr(db)),
                Condition::Let(condition_let) => is_function_call(&condition_let.expr(db)),
            };

            let is_else_if_function_call = match &else_if_condition {
                Condition::Expr(expr) => is_function_call(&expr.expr(db)),
                Condition::Let(condition_let) => is_function_call(&condition_let.expr(db)),
            };

            if is_if_function_call || is_else_if_function_call {
                return;
            }

            let if_condition_text = get_condition_text(db, &if_condition);
            let else_if_condition_text = get_condition_text(db, &else_if_condition);

            if let (Some(if_cond), Some(else_if_cond)) = (if_condition_text, else_if_condition_text) {
                if if_cond == else_if_cond {
                    diagnostics.push(PluginDiagnostic {
                        stable_ptr: if_expr.as_syntax_node().stable_ptr(),
                        message: DUPLICATE_IF_CONDITION.to_string(),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }
}
