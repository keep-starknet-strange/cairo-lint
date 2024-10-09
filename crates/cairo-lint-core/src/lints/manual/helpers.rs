use cairo_lang_defs::ids::TopLevelLanguageElementId;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Condition, Expr, ExprIf, FixedSizeArrayItems, Pattern, Statement, VarId};
use num_bigint::BigInt;

use super::is_expected_variant;
use crate::lints::{function_trait_name_from_fn_id, ARRAY_NEW, DEFAULT, FALSE};

/// Checks if the input statement is a `FunctionCall` then checks if the function name is the
/// expected function name
pub fn is_expected_function(expr: &Expr, db: &dyn SemanticGroup, func_name: &str) -> bool {
    let Expr::FunctionCall(func_call) = expr else { return false };
    func_call.function.full_name(db).as_str() == func_name
}

/// Checks if the inner_pattern in the input `Pattern::Enum` matches the given argument name.
///
/// # Arguments
/// * `pattern` - The pattern to check.
/// * `db` - Reference to the `SyntaxGroup` for syntax tree access.
/// * `arg_name` - The target name.
///
/// # Returns
/// * `true` if the argument name matches, otherwise `false`.
pub fn pattern_check_enum_arg(pattern: &Pattern, arg: &VarId, arenas: &Arenas) -> bool {
    let Pattern::EnumVariant(enum_var_pattern) = pattern else { return false };
    let Some(inner_pattern) = enum_var_pattern.inner_pattern else { return false };
    let Pattern::Variable(enum_destruct_var) = &arenas.patterns[inner_pattern] else { return false };
    let VarId::Local(expected_var) = arg else { return false };
    expected_var == &enum_destruct_var.var.id
}

/// Checks if the enum variant in the expression has the expected name and if the destructured
/// variable in the pattern is used within the expression.
///
/// This function validates two conditions for a given enum pattern and expression:
/// 1. The enum variant within the `expr` matches the `enum_name` provided.
/// 2. The destructured variable from `pattern` is used in `expr`.
///
/// # Arguments
///
/// * `expr` - A reference to an `Expr` representing the expression to check.
/// * `pattern` - A reference to a `Pattern` representing the pattern to match against.
/// * `db` - A reference to a trait object of `SemanticGroup` used for semantic analysis.
/// * `arenas` - A reference to an `Arenas` struct that provides access to allocated patterns and
///   expressions.
/// * `enum_name` - A string slice representing the expected enum variant's full path name.
///
/// # Returns
///
/// Returns `true` if:
/// - `pattern` is an enum variant pattern that matches `expr` and
/// - the destructured variable from `pattern` is used in `expr`.
///
/// Returns `false` otherwise.
///
/// # Example
///
/// Here `x` is destructured in the enum pattern and is used in the `Option::Some(x)` expression
/// ```ignore
/// match res_val {
///     Result::Ok(x) => Option::Some(x),
///     Result::Err(_) => Option::None,
/// };
/// ```
pub fn is_destructured_variable_used_and_expected_variant(
    expr: &Expr,
    pattern: &Pattern,
    db: &dyn SemanticGroup,
    arenas: &Arenas,
    enum_name: &str,
) -> bool {
    let Expr::EnumVariantCtor(enum_expr) = expr else { return false };
    if enum_expr.variant.id.full_path(db.upcast()) != enum_name {
        return false;
    };
    let Expr::Var(return_enum_var) = &arenas.exprs[enum_expr.value_expr] else { return false };
    pattern_check_enum_arg(pattern, &return_enum_var.var, arenas)
}

/// Checks if the inner pattern of a conditional `if` expression's pattern matches a block
/// statement that returns a variable associated with the destructured variable in the pattern.
///
/// This function validates whether an `if` expression contains a destructured pattern that
/// follows through the `if` block, ensuring that:
/// 1. The `if` condition pattern is an enum variant pattern with a variable as its inner pattern.
/// 2. The block within the `if` statement has a tail expression returning the same variable as the
///    one destructured in the pattern.
///
/// # Arguments
///
/// * `expr` - A reference to an `ExprIf` representing the conditional `if` expression to check.
/// * `arenas` - A reference to an `Arenas` struct that provides access to allocated patterns and
///   expressions for detailed analysis.
///
/// # Returns
///
/// Returns `true` if:
/// - The `if` condition is an enum variant pattern with an inner variable pattern.
/// - The `if` block contains a tail expression that returns the destructured variable.
///
/// Returns `false` otherwise, indicating the pattern does not match.
pub fn if_expr_pattern_matches_tail_var(expr: &ExprIf, arenas: &Arenas) -> bool {
    // Checks if it's an `if-let`
    if let Condition::Let(_condition_let, patterns) = &expr.condition
        // Checks if the pattern is an Enum pattern
        && let Pattern::EnumVariant(enum_pattern) = &arenas.patterns[patterns[0]]
        // Checks if the enum pattern has an inner pattern
        && let Some(inner_pattern) = enum_pattern.inner_pattern
        // Checks if the pattern is a variable
        && let Pattern::Variable(destruct_var) = &arenas.patterns[inner_pattern]
    {
        let Expr::Block(if_block) = &arenas.exprs[expr.if_block] else { return false };
        let Some(tail_expr) = if_block.tail else { return false };
        // Checks that the tail expression of the block is a variable.
        let Expr::Var(return_var) = &arenas.exprs[tail_expr] else { return false };
        // Checks that it's a local variable (defined in this scope)
        let VarId::Local(local_return_var) = return_var.var else { return false };
        // Checks that it's the exact variable that was created in the enum pattern
        destruct_var.var.id == local_return_var
    } else {
        false
    }
}

/// Checks if the condition pattern in an `if` expression contains an enum variant pattern
/// that matches an enum variant in the `if` block's tail expression.
///
/// This function verifies two conditions:
/// 1. The condition of the `ExprIf` expression (`expr`) contains an enum variant pattern with an
///    inner pattern.
/// 2. The tail expression in the `if` block matches the same inner pattern and corresponds to the
///    specified `enum_name`.
///
/// # Arguments
///
/// * `expr` - The `ExprIf` expression containing the enum variant pattern and the `if` block to
///   check.
/// * `db` - A reference to the `SemanticGroup`, which provides access to the syntax tree.
/// * `arenas` - A reference to the `Arenas` structure, used for accessing allocated expressions and
///   patterns.
/// * `enum_name` - The expected enum variant name to match within the `if` block's statement.
///
/// # Returns
///
/// * `true` if the inner pattern in the enum variant condition matches the first argument of the
///   enum variant with `enum_name` in the tail expression of the `if` block; otherwise, `false`.
///
/// # Example
///
/// ```ignore
/// if let EnumVariant(x) = condition {
///     EnumName(x)
/// }
/// ```
/// Checks if `x` in the condition matches `x` in the `if` block's enum pattern.
pub fn if_expr_condition_and_block_match_enum_pattern(
    expr: &ExprIf,
    db: &dyn SemanticGroup,
    arenas: &Arenas,
    enum_name: &str,
) -> bool {
    if let Condition::Let(_expr_id, patterns) = &expr.condition
        && let Pattern::EnumVariant(enum_pattern) = &arenas.patterns[patterns[0]]
        && let Some(inner_pattern) = enum_pattern.inner_pattern
        && let Pattern::Variable(variable_pattern) = &arenas.patterns[inner_pattern]
        && let Expr::Block(if_block) = &arenas.exprs[expr.if_block]
        && let Some(tail_expr_id) = if_block.tail
        && let Expr::EnumVariantCtor(enum_var) = &arenas.exprs[tail_expr_id]
        && is_expected_variant(&tail_expr_id, arenas, db, enum_name)
        && let Expr::Var(var) = &arenas.exprs[enum_var.value_expr]
        && let VarId::Local(return_var) = var.var
    {
        return_var == variable_pattern.var.id
    } else {
        false
    }
}

/// Checks if the input `Expr` is a default of the expr kind.
///
/// # Arguments
/// * `db` - Reference to the `SyntaxGroup` for syntax tree access.
/// * `expr` - The target expr.
///
/// # Returns
/// * `true` if the expression is a default otherwise `false`.
pub fn check_is_default(db: &dyn SemanticGroup, expr: &Expr, arenas: &Arenas) -> bool {
    match expr {
        Expr::FunctionCall(func_call) => {
            // Checks if the function called is either default or array new.
            let trait_name = function_trait_name_from_fn_id(db, &func_call.function);
            trait_name == DEFAULT || trait_name == ARRAY_NEW
        }
        // Empty string literal
        Expr::StringLiteral(expr_str) => expr_str.value.is_empty(),
        // If we're in a block checks that it returns default and does nothing else
        Expr::Block(expr_block) => {
            // Checks that if there is a statement in the block it's to set a variable that will be returned in
            // the tail and nothing else
            let default_subscope = if expr_block.statements.len() == 1 {
                // Check for a let assignment
                let Statement::Let(stmt) = &arenas.statements[expr_block.statements[0]] else { return false };
                let Pattern::Variable(assigned_variable) = &arenas.patterns[stmt.pattern] else { return false };

                // Checks that the tail contains a variable that is exactly the one created in the statements
                let Some(tail) = expr_block.tail else { return false };
                let Expr::Var(return_var) = &arenas.exprs[tail] else { return false };
                let VarId::Local(tail_var) = return_var.var else { return false };

                // Checks that the value assigned in the variable is a default value
                check_is_default(db, &arenas.exprs[stmt.expr], arenas) && tail_var == assigned_variable.var.id
            } else {
                false
            };
            let Some(tail_expr_id) = expr_block.tail else { return false };
            default_subscope
                || (check_is_default(db, &arenas.exprs[tail_expr_id], arenas) && expr_block.statements.is_empty())
        }
        Expr::FixedSizeArray(expr_arr) => match &expr_arr.items {
            // Case where the array is defined like that [0_u32; N]
            FixedSizeArrayItems::ValueAndSize(expr_id, _) => check_is_default(db, &arenas.exprs[*expr_id], arenas),
            // Case where the array is defined like that [0_u32, 0, 0, ...]
            FixedSizeArrayItems::Items(expr_ids) => {
                expr_ids.iter().all(|&expr| check_is_default(db, &arenas.exprs[expr], arenas))
            }
        },
        // Literal integer
        Expr::Literal(expr_literal) => expr_literal.value == BigInt::ZERO,
        // Boolean false
        Expr::EnumVariantCtor(enum_variant) => enum_variant.variant.id.full_path(db.upcast()) == FALSE,
        // Tuple contains only default elements
        Expr::Tuple(expr_tuple) => {
            expr_tuple.items.iter().all(|&expr| check_is_default(db, &arenas.exprs[expr], arenas))
        }
        _ => false,
    }
}
