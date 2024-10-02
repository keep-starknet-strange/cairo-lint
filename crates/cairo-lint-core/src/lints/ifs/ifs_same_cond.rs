use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BinaryOperator, BlockOrIf, Condition, Expr, ExprIf, OptionElseClause};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

pub const DUPLICATE_IF_CONDITION: &str = "Consecutive `if` with the same condition found.";

pub fn check_duplicate_if_condition(db: &dyn SyntaxGroup, if_expr: &ExprIf, diagnostics: &mut Vec<PluginDiagnostic>) {
    let if_condition = if_expr.condition(db);
    if matches!(&if_condition, Condition::Expr(expr) if matches!(expr.expr(db), Expr::FunctionCall(_)))
        || matches!(&if_condition, Condition::Let(condition_let) if matches!(condition_let.expr(db), Expr::FunctionCall(_)))
    {
        return;
    }

    let mut current_block = if_expr.else_clause(db);
    let mut condition_found = false;
    let if_condition_text = get_condition_text(db, &if_condition);

    while let OptionElseClause::ElseClause(ref else_clause) = current_block {
        if let BlockOrIf::If(else_if_block) = else_clause.else_block_or_if(db) {
            let else_if_condition = else_if_block.condition(db);

            if matches!(&else_if_condition, Condition::Expr(expr) if matches!(expr.expr(db), Expr::FunctionCall(_)))
                || matches!(&else_if_condition, Condition::Let(condition_let) if matches!(condition_let.expr(db), Expr::FunctionCall(_)))
            {
                current_block = else_if_block.else_clause(db);
                continue;
            }

            let else_if_condition_text = get_condition_text(db, &else_if_condition);

            if !condition_found
                && (if_condition_text == else_if_condition_text
                    || are_conditions_equivalent(&if_condition_text, &else_if_condition_text))
            {
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

fn get_condition_text(db: &dyn SyntaxGroup, condition: &Condition) -> String {
    match condition {
        Condition::Expr(expr) => {
            if let Expr::Binary(binary_expr) = expr.expr(db) {
                let operator = binary_expr.op(db);
                let lhs_text = binary_expr.lhs(db).as_syntax_node().get_text(db).to_string();
                let rhs_text = binary_expr.rhs(db).as_syntax_node().get_text(db).to_string();

                match operator {
                    BinaryOperator::Eq(_) => {
                        return format!("{} == {}", lhs_text, rhs_text);
                    }
                    BinaryOperator::LT(_) => {
                        return format!("({}, {})", rhs_text, lhs_text);
                    }
                    BinaryOperator::GT(_) => {
                        return format!("({}, {})", lhs_text, rhs_text);
                    }
                    _ => {}
                }
            }

            return expr.as_syntax_node().get_text(db).to_string();
        }
        Condition::Let(condition_let) => {
            return condition_let.expr(db).as_syntax_node().get_text(db).to_string();
        }
    }
}

fn are_conditions_equivalent(condition1: &str, condition2: &str) -> bool {
    condition1.contains("==")
        && condition2.contains("==")
        && condition1.split("==").nth(0).unwrap().trim() == condition2.split("==").nth(1).unwrap().trim()
        && condition1.split("==").nth(1).unwrap().trim() == condition2.split("==").nth(0).unwrap().trim()
}
