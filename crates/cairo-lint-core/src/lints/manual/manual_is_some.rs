use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::TypedSyntaxNode;
use cairo_lang_syntax::node::ast::{ExprMatch, Pattern};
use cairo_lang_syntax::node::db::SyntaxGroup;

pub const MANUAL_IS_SOME: &str = "Manual match for `is_some` detected. Consider using `is_some()` instead";

pub const SOME_VARIANT: &str = "Some";
pub const NONE_VARIANT: &str = "None";

pub fn check_manual_is_some(db: &dyn SyntaxGroup, expr_match: &ExprMatch, diagnostics: &mut Vec<PluginDiagnostic>) {
    let arms = expr_match.arms(db).elements(db);

    if arms.len() != 2 {
        return;
    }

    let first_arm = &arms[0];
    let second_arm = &arms[1];

    let first_pattern = &first_arm.patterns(db).elements(db)[0];
    let second_pattern = &second_arm.patterns(db).elements(db)[0];

    // Checks if the pattern matches Option::Some(_) => true
    let found_some_ok = match first_pattern {
        Pattern::Enum(enum_pattern) => {
            let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
            match enum_name.as_str() {
                "Option::Some" => first_arm.expression(db).as_syntax_node().get_text_without_trivia(db) == "true",
                _ => false,
            }
        }
        _ => false,
    };

    // Checks if the pattern matches Option::None => false
    let found_none_err = match second_pattern {
        Pattern::Enum(enum_pattern) => {
            let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
            match enum_name.as_str() {
                "Option::None" => second_arm.expression(db).as_syntax_node().get_text_without_trivia(db) == "false",
                _ => false,
            }
        }
        _ => false,
    };

    if found_some_ok && found_none_err {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_match.as_syntax_node().stable_ptr(),
            message: MANUAL_IS_SOME.to_owned(),
            severity: Severity::Warning,
        });
    }
}
