use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{Expr, ExprMatch, Pattern, PatternEnum};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

pub const MANUAL_OK_OR: &str = "Manual match for Option<T> detected. Consider using ok_or instead";

pub const OPTION_TYPE: &str = "core::option::Option::<";
pub const SOME_VARIANT: &str = "Some";
pub const NONE_VARIANT: &str = "None";

pub fn check_manual_ok_or(db: &dyn SyntaxGroup, expr_match: &ExprMatch, diagnostics: &mut Vec<PluginDiagnostic>) {
    let arms = expr_match.arms(db).elements(db);

    if arms.len() != 2 {
        return;
    }

    let first_arm = &arms[0];
    let second_arm = &arms[1];

    let found_some_ok = first_arm
        .patterns(db)
        .elements(db)
        .first()
        .and_then(|pattern| {
            if let Pattern::Enum(enum_pattern) = pattern {
                Some(check_syntax_enum_pattern(enum_pattern, first_arm.expression(db), db, SOME_VARIANT))
            } else {
                None
            }
        })
        .unwrap_or(false);

    let found_none_err = second_arm
        .patterns(db)
        .elements(db)
        .first()
        .and_then(|pattern| {
            if let Pattern::Enum(enum_pattern) = pattern {
                Some(check_syntax_enum_pattern(enum_pattern, second_arm.expression(db), db, NONE_VARIANT))
            } else {
                None
            }
        })
        .unwrap_or(false);

    if found_some_ok && found_none_err {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_match.as_syntax_node().stable_ptr(),
            message: MANUAL_OK_OR.to_owned(),
            severity: Severity::Warning,
        });
    }
}

// Check if the pattern matches Option::Some(v) => Result::Ok(v) or Option::None =>
// Result::Err('this is an err')
fn check_syntax_enum_pattern(
    enum_pattern: &PatternEnum,
    arm_expression: Expr,
    db: &dyn SyntaxGroup,
    variant_name: &str,
) -> bool {
    let path = enum_pattern.path(db);
    if path.as_syntax_node().get_text(db).starts_with(OPTION_TYPE) {
        let variant = path.as_syntax_node().get_text(db);
        if variant.contains(variant_name) {
            if let Expr::FunctionCall(func_call) = arm_expression {
                let func_name = func_call.path(db).as_syntax_node().get_text(db);
                if (variant_name == SOME_VARIANT && func_name == "Result::Ok")
                    || (variant_name == NONE_VARIANT && func_name == "Result::Err")
                {
                    return true;
                }
            }
        }
    }
    false
}
