pub mod manual_expect;
pub mod manual_is_none;
pub mod manual_is_some;
pub mod manual_ok_or;
pub mod manual_unwrap_or_default;

use cairo_lang_syntax::node::ast::{
    BlockOrIf, Condition, Expr, ExprBlock, ExprIf, ExprMatch, MatchArm, OptionElseClause,
    OptionPatternEnumInnerPattern, Pattern, Statement,
};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::kind::SyntaxKind;
use cairo_lang_syntax::node::TypedSyntaxNode;

#[derive(Copy, Clone, Debug)]
pub enum ManualLint {
    ManualOkOr,
    ManualIsSome,
    ManualIsNone,
    ManualExpect,
    ManualUnwrapOrDefault,
}

pub fn check_manual(db: &dyn SyntaxGroup, expr_match: &ExprMatch, manual_lint: ManualLint) -> bool {
    let arms = expr_match.arms(db).elements(db);

    if arms.len() != 2 {
        return false;
    }

    let first_arm = &arms[0];
    let second_arm = &arms[1];

    let found_some = match &first_arm.patterns(db).elements(db)[0] {
        Pattern::Enum(enum_pattern) => {
            let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
            match enum_name.as_str() {
                "Option::Some" => check_syntax_some_arm(first_arm, db, manual_lint),
                _ => false,
            }
        }
        _ => false,
    };

    let found_none = match &second_arm.patterns(db).elements(db)[0] {
        Pattern::Enum(enum_pattern) => {
            let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
            match enum_name.as_str() {
                "Option::None" => check_syntax_none_expression(second_arm.expression(db), db, manual_lint),
                _ => false,
            }
        }
        _ => false,
    };

    found_some && found_none
}

fn check_syntax_some_arm(arm: &MatchArm, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    match manual_lint {
        ManualLint::ManualOkOr => {
            if let Expr::FunctionCall(func_call) = arm.expression(db) {
                return func_call.path(db).as_syntax_node().get_text(db) == "Result::Ok";
            }
        }
        ManualLint::ManualIsSome => {
            return arm.expression(db).as_syntax_node().get_text_without_trivia(db) == "true";
        }
        ManualLint::ManualIsNone => {
            return arm.expression(db).as_syntax_node().get_text_without_trivia(db) == "false";
        }
        ManualLint::ManualExpect => match &arm.patterns(db).elements(db)[0] {
            Pattern::Enum(enum_pattern) => {
                let enum_arg = enum_pattern.pattern(db);
                match enum_arg {
                    OptionPatternEnumInnerPattern::PatternEnumInnerPattern(x) => {
                        return x.pattern(db).as_syntax_node().get_text_without_trivia(db)
                            == arm.expression(db).as_syntax_node().get_text_without_trivia(db);
                    }
                    OptionPatternEnumInnerPattern::Empty(_) => {
                        return false;
                    }
                }
            }
            _ => {
                return false;
            }
        },
        ManualLint::ManualUnwrapOrDefault => match &arm.patterns(db).elements(db)[0] {
            Pattern::Enum(enum_pattern) => {
                let enum_arg = enum_pattern.pattern(db);
                match enum_arg {
                    OptionPatternEnumInnerPattern::PatternEnumInnerPattern(x) => {
                        match arm.expression(db).as_syntax_node().kind(db) {
                            SyntaxKind::ExprBlock => {
                                if let Expr::Block(expr_block) = arm.expression(db) {
                                    let statement = expr_block.statements(db).elements(db)[0].clone();
                                    return statement.as_syntax_node().get_text_without_trivia(db)
                                        == x.pattern(db).as_syntax_node().get_text_without_trivia(db);
                                } else {
                                    return false;
                                }
                            }
                            SyntaxKind::ExprPath => {
                                return x.pattern(db).as_syntax_node().get_text_without_trivia(db)
                                    == arm.expression(db).as_syntax_node().get_text_without_trivia(db);
                            }
                            _ => return false,
                        }
                    }
                    OptionPatternEnumInnerPattern::Empty(_) => {
                        return false;
                    }
                }
            }
            _ => {
                return false;
            }
        },
    }
    false
}

fn check_syntax_none_expression(arm_expression: Expr, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    match manual_lint {
        ManualLint::ManualOkOr => {
            if let Expr::FunctionCall(func_call) = arm_expression {
                return func_call.path(db).as_syntax_node().get_text(db) == "Result::Err";
            }
        }
        ManualLint::ManualIsSome => {
            return arm_expression.as_syntax_node().get_text_without_trivia(db) == "false";
        }
        ManualLint::ManualIsNone => {
            return arm_expression.as_syntax_node().get_text_without_trivia(db) == "true";
        }
        ManualLint::ManualExpect => {
            if let Expr::FunctionCall(func_call) = arm_expression {
                let func_name = func_call.path(db).as_syntax_node().get_text(db);
                return func_name == "core::panic_with_felt252" || func_name == "panic_with_felt252";
            } else {
                return false;
            }
        }
        ManualLint::ManualUnwrapOrDefault => return check_is_default(db, &arm_expression),
    }

    false
}

fn check_is_default(db: &dyn SyntaxGroup, expr: &Expr) -> bool {
    match expr {
        Expr::FunctionCall(func_call) => {
            let func_name = func_call.path(db).as_syntax_node().get_text_without_trivia(db);
            func_name == "Default::default" || func_name == "ArrayTrait::new"
        }
        Expr::False(expr_false) => !expr_false.boolean_value(),
        Expr::String(expr_str) => {
            if let Some(str) = expr_str.string_value(db) {
                str.is_empty()
            } else {
                false
            }
        }
        Expr::Block(expr_block) => {
            let mut statements_check = false;
            for statement in expr_block.statements(db).elements(db) {
                statements_check = match statement {
                    Statement::Expr(statement_expr) => check_is_default(db, &statement_expr.expr(db)),
                    _ => false,
                }
            }
            statements_check
        }
        Expr::InlineMacro(expr_macro) => expr_macro.as_syntax_node().get_text_without_trivia(db) == "array![]",
        Expr::FixedSizeArray(expr_arr) => expr_arr.exprs(db).elements(db).iter().all(|expr| check_is_default(db, expr)),
        Expr::Literal(expr_literal) => expr_literal.as_syntax_node().get_text_without_trivia(db) == "0",
        Expr::Tuple(expr_tuple) => {
            expr_tuple.expressions(db).elements(db).iter().all(|expr| check_is_default(db, expr))
        }
        _ => false,
    }
}

pub fn check_manual_if(db: &dyn SyntaxGroup, expr: &ExprIf, manual_lint: ManualLint) -> bool {
    let found_some = if let Condition::Let(condition_let) = expr.condition(db) {
        match &condition_let.patterns(db).elements(db)[0] {
            Pattern::Enum(enum_pattern) => {
                let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
                match enum_name.as_str() {
                    "Option::Some" => true,
                    _ => {
                        return false;
                    }
                }
            }
            _ => {
                return false;
            }
        }
    } else {
        return false;
    };

    let found_if = check_syntax_if(expr, db, manual_lint);

    let found_else = match expr.else_clause(db) {
        OptionElseClause::Empty(_) => false,
        OptionElseClause::ElseClause(else_clase) => match else_clase.else_block_or_if(db) {
            BlockOrIf::Block(expr_block) => check_syntax_else(expr_block, db, manual_lint),
            _ => {
                return false;
            }
        },
    };

    found_some && found_if && found_else
}

fn check_syntax_if(expr: &ExprIf, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    match manual_lint {
        ManualLint::ManualOkOr => {
            let statement = expr.if_block(db).statements(db).elements(db)[0].clone();

            match statement {
                Statement::Expr(statement_expr) => {
                    let expr = statement_expr.expr(db);
                    if let Expr::FunctionCall(func_call) = expr {
                        let func_name: String =
                            func_call.path(db).as_syntax_node().get_text(db).chars().filter(|&c| c != ' ').collect();

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
        ManualLint::ManualExpect => {
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
        ManualLint::ManualUnwrapOrDefault => {
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
    }
}

fn check_syntax_else(expr_block: ExprBlock, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
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
        ManualLint::ManualExpect => {
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
        ManualLint::ManualUnwrapOrDefault => match expr_block.statements(db).elements(db)[0].clone() {
            Statement::Expr(statement_expr) => check_is_default(db, &statement_expr.expr(db)),
            _ => false,
        },
    }
}
