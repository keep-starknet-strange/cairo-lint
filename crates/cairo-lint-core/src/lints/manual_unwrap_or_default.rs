use cairo_lang_semantic::{db::SemanticGroup, Condition};
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::{Expr, Arenas};

pub const MANUAL_UNWRAP_OR_DEFAULT: &str = "This can be done in one call with `.unwrap_or_default()`";

/// Parses and extracts the branches of an `if` or `match` expression.
pub fn parse_and_extract(
    expr: &Expr,
    arenas: &Arenas
) -> Option<(Expr, Option<Expr>)> {
    match expr {
        Expr::If(expr_if) => {
            if let Condition::Let(expr_id, patterns) = &expr_if.condition {
                if patterns.len() == 1 {
                    let some_expr = arenas.exprs[*expr_id].clone();
                    // Extract the else block and return its tail expression (if present)
                    let else_expr = expr_if.else_block.and_then(|else_block_id| {
                        let else_expr = &arenas.exprs[else_block_id];
                        if let Expr::Block(expr_block) = else_expr {
                            expr_block.tail.map(|tail_expr_id| arenas.exprs[tail_expr_id].clone())
                        } else {
                            None
                        }
                    });
                    // println!("{:?}", else_expr);
                    return Some((some_expr, else_expr));
                }
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
    some_block: &Expr, 
    none_expr: &Expr
) -> bool {
    // Checks if the expression is a variable (`Expr::Var`).
    let is_var =  matches!(some_block, Expr::Var(_));

    // Checks if the given expression is `::default()` or `default`.
    let is_default =  match none_expr {
        Expr::FunctionCall(call_expr) => {
            let func_name = &call_expr.function.name(db);
            func_name.as_str().ends_with("::new\"") || func_name.as_str().ends_with("::default\"")
        },
        _ => false,
    };
    
    is_var && is_default
}

/// Detects manual `unwrap_or_default` patterns and adds a diagnostic warning if found.
pub fn check_manual_unwrap_or_default(
    db: &dyn SemanticGroup,
    expr: &Expr,
    diagnostics: &mut Vec<PluginDiagnostic>,
    arenas: &Arenas,
) {
    if let Some((some_block, Some(none_expr))) = parse_and_extract(expr, arenas) {
        if is_manual_unwrap_or_default(db, &some_block, &none_expr) {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: expr.stable_ptr().into(),
                message: MANUAL_UNWRAP_OR_DEFAULT.to_string(),
                severity: Severity::Warning,
            });
        }
    }
}
