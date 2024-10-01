pub mod helpers;
pub mod manual_expect;
pub mod manual_is_none;
pub mod manual_is_some;
pub mod manual_ok_or;

use cairo_lang_syntax::node::ast::{Condition, Expr, ExprIf, ExprMatch, MatchArm, Pattern};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;
use helpers::*;

#[derive(Copy, Clone, Debug)]
pub enum ManualLint {
    ManualOkOr,
    ManualIsSome,
    ManualIsNone,
    ManualOptExpect,
    ManualResExpect,
}

pub fn check_manual(db: &dyn SyntaxGroup, expr_match: &ExprMatch, manual_lint: ManualLint) -> bool {
    let arms = expr_match.arms(db).elements(db);

    if arms.len() != 2 {
        return false;
    }

    let first_arm = &arms[0];
    let second_arm = &arms[1];

    let found_first_arm = match &first_arm.patterns(db).elements(db)[0] {
        Pattern::Enum(enum_pattern) => {
            let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
            match enum_name.as_str() {
                "Option::Some" => check_syntax_some_arm(first_arm, db, manual_lint.clone()),
                "Result::Ok" => check_syntax_ok_arm(first_arm, db, manual_lint),
                _ => return false,
            }
        }
        _ => return false,
    };

    let found_second_arm = match &second_arm.patterns(db).elements(db)[0] {
        Pattern::Enum(enum_pattern) => {
            let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
            match enum_name.as_str() {
                "Option::None" => check_syntax_none_arm(second_arm.expression(db), db, manual_lint.clone()),
                "Result::Err" => check_syntax_err_arm(second_arm, db, manual_lint),
                _ => return false,
            }
        }
        _ => return false,
    };

    found_first_arm && found_second_arm
}

fn check_syntax_some_arm(arm: &MatchArm, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    match manual_lint {
        ManualLint::ManualOkOr => arm_expr_check_func_name(arm.expression(db), db, "Result::Ok"),
        ManualLint::ManualIsSome => arm.expression(db).as_syntax_node().get_text_without_trivia(db) == "true",
        ManualLint::ManualIsNone => arm.expression(db).as_syntax_node().get_text_without_trivia(db) == "false",
        ManualLint::ManualOptExpect => pattern_check_enum_arg(
            &arm.patterns(db).elements(db)[0],
            db,
            arm.expression(db).as_syntax_node().get_text_without_trivia(db),
        ),
        _ => false,
    }
}

fn check_syntax_ok_arm(arm: &MatchArm, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    match manual_lint {
        ManualLint::ManualResExpect => pattern_check_enum_arg(
            &arm.patterns(db).elements(db)[0],
            db,
            arm.expression(db).as_syntax_node().get_text_without_trivia(db),
        ),
        _ => false,
    }
}

fn check_syntax_none_arm(arm_expression: Expr, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    match manual_lint {
        ManualLint::ManualOkOr => arm_expr_check_func_name(arm_expression, db, "Result::Err"),
        ManualLint::ManualIsSome => arm_expression.as_syntax_node().get_text_without_trivia(db) == "false",
        ManualLint::ManualIsNone => arm_expression.as_syntax_node().get_text_without_trivia(db) == "true",
        ManualLint::ManualOptExpect => {
            if let Expr::FunctionCall(func_call) = arm_expression {
                let func_name = func_call.path(db).as_syntax_node().get_text(db);

                func_name == "core::panic_with_felt252" || func_name == "panic_with_felt252"
            } else {
                false
            }
        }
        _ => false,
    }
}

fn check_syntax_err_arm(arm: &MatchArm, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    match manual_lint {
        ManualLint::ManualResExpect => {
            if let Expr::FunctionCall(func_call) = arm.expression(db) {
                let func_name = func_call.path(db).as_syntax_node().get_text(db);

                let func_arg = pattern_check_enum_arg(&arm.patterns(db).elements(db)[0], db, "_".to_string());

                (func_name == "core::panic_with_felt252" || func_name == "panic_with_felt252") && func_arg
            } else {
                false
            }
        }
        _ => false,
    }
}

pub fn check_manual_if(db: &dyn SyntaxGroup, expr: &ExprIf, manual_lint: ManualLint) -> bool {
    let found_option = if let Condition::Let(condition_let) = expr.condition(db) {
        match &condition_let.patterns(db).elements(db)[0] {
            Pattern::Enum(enum_pattern) => {
                let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
                match enum_name.as_str() {
                    "Option::Some" => {
                        let found_if = check_syntax_opt_if(expr, db, manual_lint);

                        let found_else = check_syntax_opt_else(expr, db, manual_lint);

                        found_if && found_else
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    } else {
        false
    };

    let found_result = if let Condition::Let(condition_let) = expr.condition(db) {
        match &condition_let.patterns(db).elements(db)[0] {
            Pattern::Enum(enum_pattern) => {
                let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
                match enum_name.as_str() {
                    "Result::Ok" => {
                        let found_if = check_syntax_res_if(expr, db, manual_lint);

                        let found_else = check_syntax_res_else(expr, db, manual_lint);

                        found_if && found_else
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    } else {
        false
    };

    found_option || found_result
}

fn check_syntax_opt_if(expr: &ExprIf, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    match manual_lint {
        ManualLint::ManualOkOr => {
            statement_check_func_name(expr.if_block(db).statements(db).elements(db)[0].clone(), db, &["Result::Ok"])
        }
        ManualLint::ManualIsSome => {
            expr.if_block(db).statements(db).as_syntax_node().get_text_without_trivia(db) == "true"
        }
        ManualLint::ManualIsNone => {
            expr.if_block(db).statements(db).as_syntax_node().get_text_without_trivia(db) == "false"
        }
        ManualLint::ManualOptExpect => expr_check_inner_pattern_is_if_block_statement(expr, db),
        _ => false,
    }
}

fn check_syntax_res_if(expr: &ExprIf, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    match manual_lint {
        ManualLint::ManualResExpect => expr_check_inner_pattern_is_if_block_statement(expr, db),
        _ => false,
    }
}

fn check_syntax_opt_else(expr: &ExprIf, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    let expr_block = match get_else_expr_block(expr.else_clause(db), db) {
        Some(block) => block,
        None => return false,
    };
    match manual_lint {
        ManualLint::ManualOkOr => {
            statement_check_func_name(expr_block.statements(db).elements(db)[0].clone(), db, &["Result::Err"])
        }
        ManualLint::ManualIsSome => expr_block.statements(db).as_syntax_node().get_text_without_trivia(db) == "false",
        ManualLint::ManualIsNone => expr_block.statements(db).as_syntax_node().get_text_without_trivia(db) == "true",
        ManualLint::ManualOptExpect => statement_check_func_name(
            expr_block.statements(db).elements(db)[0].clone(),
            db,
            &["core::panic_with_felt252", "panic_with_felt252"],
        ),
        _ => false,
    }
}

fn check_syntax_res_else(expr: &ExprIf, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    let expr_block = match get_else_expr_block(expr.else_clause(db), db) {
        Some(block) => block,
        None => return false,
    };
    match manual_lint {
        ManualLint::ManualResExpect => statement_check_func_name(
            expr_block.statements(db).elements(db)[0].clone(),
            db,
            &["core::panic_with_felt252", "panic_with_felt252"],
        ),
        _ => false,
    }
}
