use cairo_lang_semantic::{db::SemanticGroup, Condition, FixedSizeArrayItems};
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::{Expr, Arenas};

use num_bigint::BigInt;

pub const MANUAL_UNWRAP_OR_DEFAULT: &str = "This can be done in one call with `.unwrap_or_default()`";

pub const DEFAULT: &str = "\"Felt252Default::default\"";
pub const ARRAY_NEW: &str = "\"ArrayImpl::new\"";


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
    arenas: &Arenas // Assuming arenas is needed for array lookup
) -> bool {
    // Check if the expression is a variable (`Expr::Var`).
    let is_var = matches!(first_arm, Expr::Var(_));
    
    // Use the helper function to check if the second expression is default
    let is_default = is_expr_default(db, second_arm, arenas);
    
    is_var && is_default
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
            func_name.as_str() == DEFAULT || func_name.as_str() == ARRAY_NEW
        },
        Expr::StringLiteral(str_expr) => {
            str_expr.value.is_empty()
        },
        Expr::Literal(int_expr) => {
            int_expr.value.eq(&BigInt::default())
        },
        Expr::EnumVariantCtor(enum_expr) => {
            match enum_expr.variant.idx {
                0 => true, // idx == 0 corresponds to `false`
                1 => false,
                _ => false,
            }
        },
        Expr::FixedSizeArray(arr_expr) => {
            match &arr_expr.items {
                // check if all items are default
                FixedSizeArrayItems::Items(items) => {
                    items.iter().all(|&expr_id| {
                        let item_expr = &arenas.exprs[expr_id];
                        is_expr_default(db, item_expr, arenas) // Recursively check each item
                    })
                },
                // If the array is repeated, check if the repeated value is default
                FixedSizeArrayItems::ValueAndSize(expr_id, _) => {
                    let item_expr = &arenas.exprs[*expr_id];
                    is_expr_default(db, item_expr, arenas) // Check the repeated value
                },
            }
        },
        Expr::Tuple(tup_expr) => {
            tup_expr.items.iter().all(|&expr_id| {
                let item_expr = &arenas.exprs[expr_id];
                is_expr_default(db, item_expr, arenas) // Recursively check each item
            })
        },
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
