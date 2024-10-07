use cairo_lang_defs::ids::TopLevelLanguageElementId;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Condition, Expr, ExprIf, FixedSizeArrayItems, Pattern, Statement, VarId};
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use num_bigint::BigInt;

use super::is_expected_variant;
use crate::lints::{function_trait_name_from_fn_id, ARRAY_NEW, DEFAULT, FALSE};

/// Checks if the input statement is a `FunctionCall` then checks if the function name is one of the
/// func_names input list.
///
/// # Arguments
/// * `statement` - A statement that may contain a `FunctionCall`.
/// * `db` - Reference to the `SyntaxGroup` for syntax tree access.
/// * `func_names` - A list of target function names.
///
/// # Returns
/// * `true` if the function name matches any of the input names, otherwise `false`.
pub fn statement_check_func_name(expr: &Expr, db: &dyn SemanticGroup, func_name: &str) -> bool {
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

/// Checks if the given `Expr::FunctionCall` expression matches the input `enum_name` and
/// verifies that the first argument of the function call corresponds to pattern enum arg.
///
/// # Arguments
/// * `expr` - The expression to check, expected to be an enum variant constructor.
/// * pattern` - The pattern to match against the enum ctor argument.
/// * `db` - Reference to the `SyntaxGroup` for accessing the syntax tree.
/// * `enum_name` - The name of the enum to match against the function call.
///
/// # Returns
/// * `true` if the expression is a the input `enum_name` enum and the first argument matches the
///   input pattern enum arg, otherwise `false`.
pub fn pattern_check_enum_arg_is_expression(
    expr: &Expr,
    pattern: &Pattern,
    db: &dyn SemanticGroup,
    arenas: &Arenas,
    enum_name: &str,
) -> bool {
    let Pattern::EnumVariant(enum_var_pattern) = pattern else { return false };
    let Some(inner_pattern) = enum_var_pattern.inner_pattern else { return false };
    let Pattern::Variable(enum_destruct_var) = &arenas.patterns[inner_pattern] else { return false };
    let Expr::EnumVariantCtor(enum_expr) = expr else { return false };
    if enum_expr.variant.id.full_path(db.upcast()) != enum_name {
        return false;
    };
    let Expr::Var(return_enum_var) = &arenas.exprs[enum_expr.value_expr] else { return false };
    let VarId::Local(return_var_id) = return_enum_var.var else { return false };
    return_var_id == enum_destruct_var.var.id
}

/// Checks that the condition expression contains an `Enum` that contains an inner pattern that is
/// the same as the statement in the if block
///
/// # Arguments
/// * `expr` - The ExprIf expression to check.
/// * `db` - Reference to the `SyntaxGroup` for syntax tree access.
///
/// # Returns
/// * `true` if the pattern matches the if block statement, otherwise `false`.
pub fn expr_check_inner_pattern_is_if_block_statement(expr: &ExprIf, arenas: &Arenas) -> bool {
    if let Condition::Let(_condition_let, patterns) = &expr.condition {
        match &arenas.patterns[patterns[0]] {
            Pattern::EnumVariant(enum_pattern) => {
                let Some(inner_patter) = enum_pattern.inner_pattern else { return false };
                match &arenas.patterns[inner_patter] {
                    Pattern::Variable(destruct_var) => {
                        let Expr::Block(if_block) = &arenas.exprs[expr.if_block] else { return false };
                        let Some(tail_expr) = if_block.tail else { return false };
                        let Expr::Var(return_var) = &arenas.exprs[tail_expr] else { return false };
                        let VarId::Local(local_return_var) = return_var.var else { return false };
                        destruct_var.var.id == local_return_var
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    } else {
        false
    }
}

/// Checks if the input `Expr` is a `FunctionCall` with the specified function name.
///
/// # Arguments
/// * `arm_expression` - The expression to check.
/// * `db` - Reference to the `SyntaxGroup` for syntax tree access.
/// * `func_name` - The target function name.
///
/// # Returns
/// * `true` if the expression is a function call and the function name matches, otherwise `false`.
pub fn arm_expr_check_func_name(arm_expression: &Expr, db: &dyn SemanticGroup, variant_path: &str) -> bool {
    if let Expr::EnumVariantCtor(variant_expr) = arm_expression {
        variant_expr.variant.id.full_path(db.upcast()) == variant_path
    } else {
        false
    }
}

/// Checks if the condition of the input `ExprIf` expression contains an enum pattern and contains
/// an inner pattern that matches the inner pattern of the enum in the if block.
///
/// # Arguments
/// * `expr` - The `ExprIf` expression containing the condition and the if block to check.
/// * `db` - Reference to the `SyntaxGroup` for accessing the syntax tree.
/// * `enum_name` - The name of the enum to match against the function call in the if block.
///
/// # Example
/// for :
/// if let Enum(x) = res_val {
///     EnumName(x)
/// }
/// checks that x == x
///
/// # Returns
/// * `true` if the inner pattern of the enum in the condition matches the first argument of the
///   enum `enume_name` in the statement of the if block, otherwise `false`.
pub fn expr_check_condition_enum_inner_pattern_is_if_block_enum_inner_pattern(
    expr: &ExprIf,
    db: &dyn SemanticGroup,
    arenas: &Arenas,
    enum_name: &str,
) -> bool {
    let Condition::Let(_expr_id, patterns) = &expr.condition else {
        return false;
    };

    let Pattern::EnumVariant(enum_pattern) = &arenas.patterns[patterns[0]] else {
        return false;
    };

    let Some(inner_pattern) = enum_pattern.inner_pattern else {
        return false;
    };
    let Pattern::Variable(variable_pattern) = &arenas.patterns[inner_pattern] else {
        return false;
    };

    let Expr::Block(if_block) = &arenas.exprs[expr.if_block] else {
        return false;
    };
    let Some(tail_expr_id) = if_block.tail else { return false };

    let Expr::EnumVariantCtor(enum_var) = &arenas.exprs[tail_expr_id] else {
        return false;
    };

    if !is_expected_variant(&tail_expr_id, arenas, db, enum_name) {
        return false;
    }

    let Expr::Var(var) = &arenas.exprs[enum_var.value_expr] else { return false };
    var.stable_ptr.lookup(db.upcast()).as_syntax_node().get_text_without_trivia(db.upcast()) == variable_pattern.name
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
            let trait_name = function_trait_name_from_fn_id(db, &func_call.function);
            trait_name == DEFAULT || trait_name == ARRAY_NEW
        }
        Expr::StringLiteral(expr_str) => expr_str.value.is_empty(),
        Expr::Block(expr_block) => {
            let default_subscope = if expr_block.statements.len() == 1 {
                let Statement::Let(stmt) = &arenas.statements[expr_block.statements[0]] else { return false };
                let Pattern::Variable(assigned_variable) = &arenas.patterns[stmt.pattern] else { return false };

                let Some(tail) = expr_block.tail else { return false };
                let Expr::Var(return_var) = &arenas.exprs[tail] else { return false };
                let VarId::Local(tail_var) = return_var.var else { return false };

                check_is_default(db, &arenas.exprs[stmt.expr], arenas) && tail_var == assigned_variable.var.id
            } else {
                false
            };
            let Some(tail_expr_id) = expr_block.tail else { return false };
            default_subscope || check_is_default(db, &arenas.exprs[tail_expr_id], arenas)
        }
        Expr::FixedSizeArray(expr_arr) => match &expr_arr.items {
            FixedSizeArrayItems::ValueAndSize(expr_id, _) => check_is_default(db, &arenas.exprs[*expr_id], arenas),
            FixedSizeArrayItems::Items(expr_ids) => {
                expr_ids.iter().all(|&expr| check_is_default(db, &arenas.exprs[expr], arenas))
            }
        },
        Expr::Literal(expr_literal) => expr_literal.value == BigInt::ZERO,
        Expr::EnumVariantCtor(enum_variant) => enum_variant.variant.id.full_path(db.upcast()) == FALSE,
        Expr::Tuple(expr_tuple) => {
            expr_tuple.items.iter().all(|&expr| check_is_default(db, &arenas.exprs[expr], arenas))
        }
        _ => false,
    }
}
