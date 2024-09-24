use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{Expr, ExprMatch, Pattern};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

pub const MANUAL_OK_OR: &str = "Manual match for Option<T> detected. Consider using ok_or instead";

pub const SOME_VARIANT: &str = "Some";
pub const NONE_VARIANT: &str = "None";

pub fn check_manual_ok_or(db: &dyn SyntaxGroup, expr_match: &ExprMatch, diagnostics: &mut Vec<PluginDiagnostic>) {
    let arms = expr_match.arms(db).elements(db);

    if arms.len() != 2 {
        return;
    }

    let first_arm = &arms[0];
    let second_arm = &arms[1];

    let first_pattern = &first_arm.patterns(db).elements(db)[0];
    let second_pattern = &second_arm.patterns(db).elements(db)[0];

    // Checks if the pattern matches Option::Some(v) => Result::Ok(v)
    let found_some_ok = match first_pattern {
        Pattern::Enum(enum_pattern) => {
            let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
            match enum_name.as_str() {
                "Option::Some" => check_syntax_enum_pattern(first_arm.expression(db), db, SOME_VARIANT),
                _ => false,
            }
        }
        _ => false,
    };

    // Checks if the pattern matches Option::None => Result::Err('this is an err')
    let found_none_err = match second_pattern {
        Pattern::Enum(enum_pattern) => {
            let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
            match enum_name.as_str() {
                "Option::None" => check_syntax_enum_pattern(second_arm.expression(db), db, NONE_VARIANT),
                _ => false,
            }
        }
        _ => false,
    };

    if found_some_ok && found_none_err {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_match.as_syntax_node().stable_ptr(),
            message: MANUAL_OK_OR.to_owned(),
            severity: Severity::Warning,
        });
    }
}

fn check_syntax_enum_pattern(arm_expression: Expr, db: &dyn SyntaxGroup, variant_name: &str) -> bool {
    if let Expr::FunctionCall(func_call) = arm_expression {
        let func_name = func_call.path(db).as_syntax_node().get_text(db);
        if (variant_name == SOME_VARIANT && func_name == "Result::Ok")
            || (variant_name == NONE_VARIANT && func_name == "Result::Err")
        {
            return true;
        }
    }

    false
}
