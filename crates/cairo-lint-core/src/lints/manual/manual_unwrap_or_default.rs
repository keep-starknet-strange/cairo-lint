use cairo_lang_semantic::{db::SemanticGroup, Condition, ExprBlock, FixedSizeArrayItems, Statement};
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::{Expr, Arenas};
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use num_bigint::BigInt;

pub const MANUAL_UNWRAP_OR_DEFAULT: &str = "This can be done in one call with `.unwrap_or_default()`";

pub const DEFAULT: &str = "Default::default";
pub const ARRAY_NEW: &str = "\"ArrayImpl::new\"";
pub const FALSE: &str = "#[default]\n    False";

/// Parses and extracts the branches of an `if` or `match` expression.
pub fn parse_and_extract(
    expr: &Expr,
    arenas: &Arenas
) -> Option<(Expr, Option<Expr>)> {
    match expr {
        Expr::If(expr_if) => {
            if let Condition::Let(expr_id, patterns) = &expr_if.condition {
                // Return early if the pattern length is not 1
                if patterns.len() != 1 {
                    return None;
                }
                let some_expr = arenas.exprs[*expr_id].clone();
                // Extract the else block and return its tail expression (if present)
                let else_expr = expr_if.else_block.and_then(|else_block_id| {
                    let else_expr = &arenas.exprs[else_block_id];
                    match else_expr {
                        Expr::Block(expr_block) => expr_block
                            .tail
                            .map(|tail_expr_id| arenas.exprs[tail_expr_id].clone()),
                        _ => None,
                    }
                });
                return Some((some_expr, else_expr));
            }
            None
        },
        Expr::Match(expr_match) => {
            let arms = &expr_match.arms;
            if arms.len() == 2 {
                let some_arm_expr = &arenas.exprs[arms[0].expression];
                let none_arm_expr = &arenas.exprs[arms[1].expression];
                Some((some_arm_expr.clone(), Some(none_arm_expr.clone())))
            } else {
                None
            }
        },
        _ => None,
    }
}
/// Checks if the pattern is `Some(x) => x` and the other arm is `Default::default()`.
fn is_manual_unwrap_or_default(
    db: &dyn SemanticGroup, 
    first_arm: &Expr, 
    second_arm: &Expr,
    arenas: &Arenas
) -> bool {
    is_expr_var(db, first_arm, arenas) && is_expr_default(db, second_arm, arenas)
}

fn is_expr_var(
    db: &dyn SemanticGroup, 
    expr: &Expr,
    arenas: &Arenas
) -> bool {
    match expr {
        Expr::Var(_) => true,
        Expr::Block(block_expr) => check_block_expr(db, block_expr, arenas, is_expr_var),
        _ => false   
    }
}

/// Helper function to check if an expression is a "default" value
fn is_expr_default(
    db: &dyn SemanticGroup, 
    expr: &Expr,
    arenas: &Arenas
) -> bool {
    match expr {
        Expr::FunctionCall(call_expr) => {
            let func_name = &call_expr.function.name(db);
            func_name.as_str() == DEFAULT || func_name.as_str() == ARRAY_NEW || func_name.as_str().contains(DEFAULT)
        },
        Expr::StringLiteral(str_expr) => {
            str_expr.value.is_empty()
        },
        Expr::Literal(int_expr) => {
            int_expr.value.eq(&BigInt::default())
        },
        Expr::EnumVariantCtor(enum_expr) => {
            let enum_text = enum_expr.variant.id
                .stable_ptr(db.upcast())
                .lookup(db.upcast())
                .as_syntax_node().get_text_without_trivia(db.upcast());
            enum_text == FALSE
        },
        Expr::Block(block_expr) => check_block_expr(db, block_expr, arenas, is_expr_default),
        Expr::FixedSizeArray(arr_expr) => {
            match &arr_expr.items {
                // check if all items are default
                FixedSizeArrayItems::Items(items) => {
                    items.iter().all(|&expr_id| is_expr_default(db, &arenas.exprs[expr_id], arenas))
                },
                // If the array is repeated, check if the repeated value is default
                FixedSizeArrayItems::ValueAndSize(expr_id, _) => {
                    is_expr_default(db, &arenas.exprs[*expr_id], arenas)
                },
            }
        },
        Expr::Tuple(tup_expr) => {
            tup_expr.items.iter().all(|&expr_id| is_expr_default(db, &arenas.exprs[expr_id], arenas))
        },
        _ => false,
    }
}

/// Helper function for checking block expressions.
fn check_block_expr<F>(
    db: &dyn SemanticGroup,
    block_expr: &ExprBlock,
    arenas: &Arenas,
    checker: F,
) -> bool
where
    F: Fn(&dyn SemanticGroup, &Expr, &Arenas) -> bool,
{
    let statements_default = if !block_expr.statements.is_empty() {
        block_expr.statements.iter().all(|&statement_id| {
            let statement = &arenas.statements[statement_id];
            match statement {
                Statement::Let(statement_let) => {
                    let expr = &arenas.exprs[statement_let.expr];
                    checker(db, expr, arenas)
                }
                _ => false,
            }
        })
    } else {
        false
    };

    // Check the tail expression, if it exists
    let tail_default = block_expr.tail.map_or(false, |tail_expr_id| {
        let tail_expr = &arenas.exprs[tail_expr_id];
        checker(db, tail_expr, arenas)
    });

    statements_default || tail_default
}

/// Detects manual `unwrap_or_default` patterns and adds a diagnostic warning if found.
pub fn check_manual_unwrap_or_default(
    db: &dyn SemanticGroup,
    expr: &Expr,
    diagnostics: &mut Vec<PluginDiagnostic>,
    arenas: &Arenas,
) {
    if let Some((first_arm, Some(second_arm))) = parse_and_extract(expr, arenas) {
        if is_manual_unwrap_or_default(db, &first_arm, &second_arm, arenas) {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: expr.stable_ptr().into(),
                message: MANUAL_UNWRAP_OR_DEFAULT.to_string(),
                severity: Severity::Warning,
            });
        }
    }
}
