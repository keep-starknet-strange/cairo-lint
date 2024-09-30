use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{ExprMatch, Pattern};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

pub const MANUAL_UNWRAP_OR: &str =
    "Manual match for Option<T> or Result<T, E> detected. Consider using unwrap_or instead.";

pub fn check_manual_unwrap_or(db: &dyn SyntaxGroup, expr_match: &ExprMatch, diagnostics: &mut Vec<PluginDiagnostic>) {
    let arms = expr_match.arms(db).elements(db);

    if arms.len() != 2 {
        return;
    }

    let first_arm = &arms[0];
    let second_arm = &arms[1];

    let first_pattern = &first_arm.patterns(db).elements(db)[0];
    let second_pattern = &second_arm.patterns(db).elements(db)[0];

    // Check for Option::Some and Option::None
    let found_option_some_none = is_option_some_none(db, first_pattern, second_pattern);

    // Check for Result::Ok and Result::Err
    let found_result_ok_err = is_result_ok_err(db, first_pattern, second_pattern);

    // If both patterns are Some/None or Ok/Err, generate diagnosis
    if found_option_some_none || found_result_ok_err {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_match.as_syntax_node().stable_ptr(),
            message: MANUAL_UNWRAP_OR.to_string(),
            severity: Severity::Warning,
        });
    }
}

// Check if the patterns are Option::Some and Option::None.
fn is_option_some_none(db: &dyn SyntaxGroup, first_pattern: &Pattern, second_pattern: &Pattern) -> bool {
    let found_some = match first_pattern {
        Pattern::Enum(enum_pattern) => {
            let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
            enum_name.as_str() == "Option::Some"
        }
        _ => false,
    };

    let found_none = match second_pattern {
        Pattern::Enum(enum_pattern) => {
            let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
            enum_name.as_str() == "Option::None"
        }
        _ => false,
    };

    found_some && found_none
}

// Check if the patterns are Result::Ok and Result::Err.
fn is_result_ok_err(db: &dyn SyntaxGroup, first_pattern: &Pattern, second_pattern: &Pattern) -> bool {
    let found_ok = match first_pattern {
        Pattern::Enum(enum_pattern) => {
            let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
            enum_name.as_str() == "Result::Ok"
        }
        _ => false,
    };

    let found_err = match second_pattern {
        Pattern::Enum(enum_pattern) => {
            let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
            enum_name.as_str() == "Result::Err"
        }
        _ => false,
    };

    found_ok && found_err
}
