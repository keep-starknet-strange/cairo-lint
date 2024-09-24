use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BlockOrIf, Condition, Expr, ExprIf, OptionElseClause};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

pub const DUPLICATE_IF_CONDITION: &str = "Consecutive `if` with the same condition found.";

fn get_condition_text(db: &dyn SyntaxGroup, condition: &Condition) -> String {
    match condition {
        Condition::Expr(expr) => expr.as_syntax_node().get_text(db).to_string(),
        Condition::Let(condition_let) => condition_let.expr(db).as_syntax_node().get_text(db).to_string(),
    }
}

pub fn check_duplicate_if_condition(db: &dyn SyntaxGroup, if_expr: &ExprIf, diagnostics: &mut Vec<PluginDiagnostic>) {
    let if_condition = if_expr.condition(db);

    if matches!(&if_condition, Condition::Expr(expr) if matches!(expr.expr(db), Expr::FunctionCall(_)))
        || matches!(&if_condition, Condition::Let(condition_let) if matches!(condition_let.expr(db), Expr::FunctionCall(_)))
    {
        return;
    }

    let mut current_block = if_expr.else_clause(db);
    let mut condition_found = false;

    while let OptionElseClause::ElseClause(ref else_clause) = current_block {
        if let BlockOrIf::If(else_if_block) = else_clause.else_block_or_if(db) {
            let else_if_condition = else_if_block.condition(db);

            if matches!(&else_if_condition, Condition::Expr(expr) if matches!(expr.expr(db), Expr::FunctionCall(_)))
                || matches!(&else_if_condition, Condition::Let(condition_let) if matches!(condition_let.expr(db), Expr::FunctionCall(_)))
            {
                current_block = else_if_block.else_clause(db);
                continue;
            }

            let if_condition_text = get_condition_text(db, &if_condition);
            let else_if_condition_text = get_condition_text(db, &else_if_condition);

            if if_condition_text == else_if_condition_text && !condition_found {
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: if_expr.as_syntax_node().stable_ptr(),
                    message: DUPLICATE_IF_CONDITION.to_string(),
                    severity: Severity::Warning,
                });
                condition_found = true;
            }

            current_block = else_if_block.else_clause(db);
        } else {
            break;
        }
    }
}
