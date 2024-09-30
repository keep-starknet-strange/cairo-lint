use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Condition, Expr, FixedSizeArrayItems, Statement};
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use num_bigint::BigInt;

pub const MANUAL_UNWRAP_OR_DEFAULT: &str = "This can be done in one call with `.unwrap_or_default()`";

pub const DEFAULT: &str = "Default::default";
pub const ARRAY_NEW: &str = "\"ArrayImpl::new\"";
pub const FALSE: &str = "#[default]\n    False";

/// Parses and extracts the branches of an `if` or `match` expression.
pub fn parse_and_extract(expr: &Expr, arenas: &Arenas) -> Option<(Expr, Option<Expr>, Option<Expr>)> {
    match expr {
        Expr::If(expr_if) => {
            let if_expr = match &arenas.exprs[expr_if.if_block] {
                // only interested in the return value.
                Expr::Block(expr_block) => expr_block.tail.map(|tail_expr_id| arenas.exprs[tail_expr_id].clone()),
                _ => None,
            };

            if let Condition::Let(expr_id, patterns) = &expr_if.condition {
                // Return early if the pattern length is not 1
                if patterns.len() != 1 {
                    return None;
                }
                let match_expr = arenas.exprs[*expr_id].clone();
                // Extract the else block and return its tail expression (if present)
                let else_expr = expr_if.else_block.and_then(|else_block_id| {
                    let else_expr = &arenas.exprs[else_block_id];
                    match else_expr {
                        Expr::Block(expr_block) => {
                            expr_block.tail.map(|tail_expr_id| arenas.exprs[tail_expr_id].clone())
                        }
                        _ => None,
                    }
                });
                return Some((match_expr, if_expr, else_expr));
            }
            None
        }
        Expr::Match(expr_match) => {
            let match_expr_id = &expr_match.matched_expr;
            let match_expr = &arenas.exprs[*match_expr_id];
            let arms = &expr_match.arms;
            if arms.len() == 2 {
                let some_arm_expr = &arenas.exprs[arms[0].expression];
                let none_arm_expr = &arenas.exprs[arms[1].expression];
                Some((match_expr.clone(), Some(some_arm_expr.clone()), Some(none_arm_expr.clone())))
            } else {
                None
            }
        }
        _ => None,
    }
}
/// Checks if the pattern is `Some(x) => x` and the other arm is `Default::default()`.
fn is_manual_unwrap_or_default(
    db: &dyn SemanticGroup,
    match_arm: &Expr,
    first_arm: &Expr,
    second_arm: &Expr,
    arenas: &Arenas,
) -> bool {
    is_expr_var(db, match_arm, first_arm, arenas) && is_expr_default(db, second_arm, arenas)
}

fn is_expr_var(db: &dyn SemanticGroup, match_arm: &Expr, first_arm: &Expr, arenas: &Arenas) -> bool {
    match first_arm {
        Expr::Var(_) => {
            let first_arm_type = first_arm.ty().format(db);
            let match_arm_type = match_arm.ty().format(db);
            match_arm_type.contains(&first_arm_type)
        }
        Expr::Block(block_expr) => block_expr.tail.map_or(false, |tail_expr_id| {
            let tail_expr = &arenas.exprs[tail_expr_id];
            is_expr_var(db, match_arm, tail_expr, arenas)
        }),
        _ => false,
    }
}

/// Helper function to check if an expression is a "default" value
fn is_expr_default(db: &dyn SemanticGroup, second_arm: &Expr, arenas: &Arenas) -> bool {
    match second_arm {
        Expr::FunctionCall(call_expr) => {
            let func_name = &call_expr.function.name(db);
            func_name.as_str() == DEFAULT || func_name.as_str() == ARRAY_NEW || func_name.as_str().contains(DEFAULT)
        }
        Expr::StringLiteral(str_expr) => str_expr.value.is_empty(),
        Expr::Literal(int_expr) => int_expr.value.eq(&BigInt::default()),
        Expr::EnumVariantCtor(enum_expr) => {
            let enum_text = enum_expr
                .variant
                .id
                .stable_ptr(db.upcast())
                .lookup(db.upcast())
                .as_syntax_node()
                .get_text_without_trivia(db.upcast());
            enum_text == FALSE
        }
        Expr::Block(block_expr) => {
            // Initialize flag
            let mut statements_check = false;

            // check statements in block
            for &statement_id in &block_expr.statements {
                let statement = &arenas.statements[statement_id];
                statements_check = match statement {
                    Statement::Let(statement_let) => {
                        let expr = &arenas.exprs[statement_let.expr];
                        is_expr_default(db, expr, arenas)
                    }
                    _ => false,
                }
            }

            let is_tail_default = block_expr.tail.map_or(false, |tail_expr_id| {
                let tail_expr = &arenas.exprs[tail_expr_id];
                is_expr_default(db, tail_expr, arenas)
            });

            statements_check || is_tail_default
        }
        Expr::FixedSizeArray(arr_expr) => {
            match &arr_expr.items {
                // check if all items are default
                FixedSizeArrayItems::Items(items) => {
                    items.iter().all(|&expr_id| is_expr_default(db, &arenas.exprs[expr_id], arenas))
                }
                // If the array is repeated, check if the repeated value is default
                FixedSizeArrayItems::ValueAndSize(expr_id, _) => is_expr_default(db, &arenas.exprs[*expr_id], arenas),
            }
        }
        Expr::Tuple(tup_expr) => {
            tup_expr.items.iter().all(|&expr_id| is_expr_default(db, &arenas.exprs[expr_id], arenas))
        }
        _ => false,
    }
}

/// Detects manual `unwrap_or_default` patterns and adds a diagnostic warning if found.
pub fn check_manual_unwrap_or_default(
    db: &dyn SemanticGroup,
    expr: &Expr,
    diagnostics: &mut Vec<PluginDiagnostic>,
    arenas: &Arenas,
) {
    if let Some((match_arm, Some(first_arm), Some(second_arm))) = parse_and_extract(expr, arenas) {
        if is_manual_unwrap_or_default(db, &match_arm, &first_arm, &second_arm, arenas) {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: expr.stable_ptr().into(),
                message: MANUAL_UNWRAP_OR_DEFAULT.to_string(),
                severity: Severity::Warning,
            });
        }
    }
}
