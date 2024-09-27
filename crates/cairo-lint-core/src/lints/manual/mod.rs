pub mod manual_assert;
pub mod manual_expect;
pub mod manual_is_none;
pub mod manual_is_some;
pub mod manual_ok_or;

use cairo_lang_syntax::node::ast::{Expr, ExprMatch, MatchArm, OptionPatternEnumInnerPattern, Pattern};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

#[derive(Copy, Clone, Debug)]
pub enum ManualLint {
    ManualOkOr,
    ManualIsSome,
    ManualIsNone,
    ManualExpect,
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
    }

    false
}
