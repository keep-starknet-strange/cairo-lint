use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{Expr, ExprBlock, ExprListParenthesized, ExprMatch, Pattern, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const DESTRUCT_MATCH: &str =
    "you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`";
pub const MATCH_FOR_EQUALITY: &str = "you seem to be trying to use `match` for an equality check. Consider using `if`";

fn tuple_expr_in_block_expr(
    db: &dyn SyntaxGroup,
    block_expr: &ExprBlock,
    is_single_armed: &mut bool,
) -> Option<ExprListParenthesized> {
    let statements = block_expr.statements(db).elements(db);
    if statements.is_empty() {
        *is_single_armed = true;
    }
    if statements.len() == 1
        && let Statement::Expr(statement_expr) = &statements[0]
        && let Expr::Tuple(tuple_expr) = statement_expr.expr(db)
    {
        Some(tuple_expr)
    } else {
        None
    }
}

pub fn check_single_match(db: &dyn SyntaxGroup, match_expr: &ExprMatch, diagnostics: &mut Vec<PluginDiagnostic>) {
    let arms = match_expr.arms(db).elements(db);
    let mut is_single_armed = false;
    let mut is_destructuring = false;
    if arms.len() == 2 {
        for arm in arms {
            let patterns = arm.patterns(db).elements(db);
            match &patterns[0] {
                Pattern::Underscore(_) => {
                    let tuple_expr = match arm.expression(db) {
                        Expr::Block(block_expr) => tuple_expr_in_block_expr(db, &block_expr, &mut is_single_armed),
                        Expr::Tuple(tuple_expr) => Some(tuple_expr),
                        _ => None,
                    };
                    is_single_armed =
                        tuple_expr.is_some_and(|list| list.expressions(db).elements(db).is_empty()) || is_single_armed;
                }
                Pattern::Enum(_) | Pattern::Struct(_) => {
                    is_destructuring = true;
                }
                _ => (),
            };
        }
    };
    match (is_single_armed, is_destructuring) {
        (true, false) => diagnostics.push(PluginDiagnostic {
            stable_ptr: match_expr.stable_ptr().untyped(),
            message: MATCH_FOR_EQUALITY.to_string(),
            severity: Severity::Warning,
        }),
        (true, true) => diagnostics.push(PluginDiagnostic {
            stable_ptr: match_expr.stable_ptr().untyped(),
            message: DESTRUCT_MATCH.to_string(),
            severity: Severity::Warning,
        }),
        (_, _) => (),
    }
}
