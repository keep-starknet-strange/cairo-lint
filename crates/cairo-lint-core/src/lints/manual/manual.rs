use cairo_lang_syntax::node::ast::{Expr, ExprMatch, Pattern};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

#[derive(Copy, Clone, Debug)]
pub enum ManualLint {
    ManualOkOr,
    ManualIsNone,
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
                "Option::Some" => check_syntax_some_expression(first_arm.expression(db), db, manual_lint),
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

fn check_syntax_some_expression(arm_expression: Expr, db: &dyn SyntaxGroup, manual_lint: ManualLint) -> bool {
    match manual_lint {
        ManualLint::ManualOkOr => {
            if let Expr::FunctionCall(func_call) = arm_expression {
                return func_call.path(db).as_syntax_node().get_text(db) == "Result::Ok";
            }
        }
        ManualLint::ManualIsNone => {
            return arm_expression.as_syntax_node().get_text_without_trivia(db) == "false";
        }
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
        ManualLint::ManualIsNone => {
            return arm_expression.as_syntax_node().get_text_without_trivia(db) == "true";
        }
    }
    false
}

