pub mod manual_expect;
pub mod manual_is_none;
pub mod manual_is_some;
pub mod manual_ok_or;

use cairo_lang_syntax::node::ast::{
    BlockOrIf, Condition, Expr, ExprIf, ExprMatch, MatchArm, OptionElseClause, OptionPatternEnumInnerPattern, Pattern,
    Statement,
};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

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
        ManualLint::ManualOkOr => {
            if let Expr::FunctionCall(func_call) = arm.expression(db) {
                func_call.path(db).as_syntax_node().get_text(db) == "Result::Ok"
            } else {
                false
            }
        }
        ManualLint::ManualIsSome => arm.expression(db).as_syntax_node().get_text_without_trivia(db) == "true",
        ManualLint::ManualIsNone => arm.expression(db).as_syntax_node().get_text_without_trivia(db) == "false",
        ManualLint::ManualOptExpect => match &arm.patterns(db).elements(db)[0] {
            Pattern::Enum(enum_pattern) => {
                let enum_arg = enum_pattern.pattern(db);
                match enum_arg {
                    OptionPatternEnumInnerPattern::PatternEnumInnerPattern(x) => {
                        x.pattern(db).as_syntax_node().get_text_without_trivia(db)
                            == arm.expression(db).as_syntax_node().get_text_without_trivia(db)
                    }
                    OptionPatternEnumInnerPattern::Empty(_) => false,
                }
            }
            _ => false,
        },
        _ => false,
    }
}

fn check_syntax_ok_arm(arm: &MatchArm, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    match manual_lint {
        ManualLint::ManualResExpect => match &arm.patterns(db).elements(db)[0] {
            Pattern::Enum(enum_pattern) => {
                let enum_arg = enum_pattern.pattern(db);
                match enum_arg {
                    OptionPatternEnumInnerPattern::PatternEnumInnerPattern(x) => {
                        x.pattern(db).as_syntax_node().get_text_without_trivia(db)
                            == arm.expression(db).as_syntax_node().get_text_without_trivia(db)
                    }
                    OptionPatternEnumInnerPattern::Empty(_) => false,
                }
            }
            _ => false,
        },
        _ => false,
    }
}

fn check_syntax_none_arm(arm_expression: Expr, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    match manual_lint {
        ManualLint::ManualOkOr => {
            if let Expr::FunctionCall(func_call) = arm_expression {
                func_call.path(db).as_syntax_node().get_text(db) == "Result::Err"
            } else {
                false
            }
        }
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

                let func_arg = match &arm.patterns(db).elements(db)[0] {
                    Pattern::Enum(enum_pattern) => {
                        let enum_arg = enum_pattern.pattern(db);
                        match enum_arg {
                            OptionPatternEnumInnerPattern::PatternEnumInnerPattern(inner_pattern) => {
                                inner_pattern.pattern(db).as_syntax_node().get_text_without_trivia(db)
                            }
                            OptionPatternEnumInnerPattern::Empty(_) => {
                                return false;
                            }
                        }
                    }
                    _ => {
                        return false;
                    }
                };

                (func_name == "core::panic_with_felt252" || func_name == "panic_with_felt252") && func_arg == "_"
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
            let statement = expr.if_block(db).statements(db).elements(db)[0].clone();

            match statement {
                Statement::Expr(statement_expr) => {
                    let expr = statement_expr.expr(db);
                    if let Expr::FunctionCall(func_call) = expr {
                        let func_name: String = func_call.path(db).as_syntax_node().get_text_without_trivia(db);

                        func_name == "Result::Ok"
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
        ManualLint::ManualIsSome => {
            expr.if_block(db).statements(db).as_syntax_node().get_text_without_trivia(db) == "true"
        }
        ManualLint::ManualIsNone => {
            expr.if_block(db).statements(db).as_syntax_node().get_text_without_trivia(db) == "false"
        }
        ManualLint::ManualOptExpect => {
            if let Condition::Let(condition_let) = expr.condition(db) {
                match &condition_let.patterns(db).elements(db)[0] {
                    Pattern::Enum(enum_pattern) => {
                        let enum_arg = enum_pattern.pattern(db);
                        match enum_arg {
                            OptionPatternEnumInnerPattern::PatternEnumInnerPattern(inner_pattern) => {
                                inner_pattern.pattern(db).as_syntax_node().get_text_without_trivia(db)
                                    == expr.if_block(db).statements(db).as_syntax_node().get_text_without_trivia(db)
                            }
                            OptionPatternEnumInnerPattern::Empty(_) => false,
                        }
                    }
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

fn check_syntax_res_if(expr: &ExprIf, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    match manual_lint {
        ManualLint::ManualResExpect => {
            if let Condition::Let(condition_let) = expr.condition(db) {
                match &condition_let.patterns(db).elements(db)[0] {
                    Pattern::Enum(enum_pattern) => {
                        let enum_arg = enum_pattern.pattern(db);
                        match enum_arg {
                            OptionPatternEnumInnerPattern::PatternEnumInnerPattern(inner_pattern) => {
                                inner_pattern.pattern(db).as_syntax_node().get_text_without_trivia(db)
                                    == expr.if_block(db).statements(db).as_syntax_node().get_text_without_trivia(db)
                            }
                            OptionPatternEnumInnerPattern::Empty(_) => false,
                        }
                    }
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

fn check_syntax_opt_else(expr: &ExprIf, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    let expr_block = match expr.else_clause(db) {
        OptionElseClause::Empty(_) => {
            return false;
        }
        OptionElseClause::ElseClause(else_clase) => match else_clase.else_block_or_if(db) {
            BlockOrIf::Block(expr_block) => expr_block,
            _ => {
                return false;
            }
        },
    };
    match manual_lint {
        ManualLint::ManualOkOr => {
            let statement = expr_block.statements(db).elements(db)[0].clone();

            match statement {
                Statement::Expr(statement_expr) => {
                    let expr = statement_expr.expr(db);
                    if let Expr::FunctionCall(func_call) = expr {
                        let func_name: String = func_call.path(db).as_syntax_node().get_text_without_trivia(db);
                        func_name == "Result::Err"
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
        ManualLint::ManualIsSome => expr_block.statements(db).as_syntax_node().get_text_without_trivia(db) == "false",
        ManualLint::ManualIsNone => expr_block.statements(db).as_syntax_node().get_text_without_trivia(db) == "true",
        ManualLint::ManualOptExpect => {
            let statement = expr_block.statements(db).elements(db)[0].clone();

            match statement {
                Statement::Expr(statement_expr) => {
                    let expr = statement_expr.expr(db);
                    if let Expr::FunctionCall(func_call) = expr {
                        let func_name: String = func_call.path(db).as_syntax_node().get_text_without_trivia(db);
                        func_name == "core::panic_with_felt252" || func_name == "panic_with_felt252"
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
        _ => false,
    }
}

fn check_syntax_res_else(expr: &ExprIf, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    let expr_block = match expr.else_clause(db) {
        OptionElseClause::Empty(_) => {
            return false;
        }
        OptionElseClause::ElseClause(else_clase) => match else_clase.else_block_or_if(db) {
            BlockOrIf::Block(expr_block) => expr_block,
            _ => {
                return false;
            }
        },
    };
    match manual_lint {
        ManualLint::ManualResExpect => {
            let statement = expr_block.statements(db).elements(db)[0].clone();

            match statement {
                Statement::Expr(statement_expr) => {
                    let expr = statement_expr.expr(db);
                    if let Expr::FunctionCall(func_call) = expr {
                        let func_name: String = func_call.path(db).as_syntax_node().get_text_without_trivia(db);

                        func_name == "core::panic_with_felt252" || func_name == "panic_with_felt252"
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
        _ => false,
    }
}
