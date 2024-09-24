use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::TypedSyntaxNode;
use cairo_lang_syntax::node::ast::{Expr, ExprMatch, OptionPatternEnumInnerPattern, Pattern};
use cairo_lang_syntax::node::db::SyntaxGroup;

pub const MANUAL_EXPECT: &str = "Manual match for expect detected. Consider using `expect()` instead";

pub fn check_manual_expect(db: &dyn SyntaxGroup, expr_match: &ExprMatch, diagnostics: &mut Vec<PluginDiagnostic>) {
    let arms = expr_match.arms(db).elements(db);

    if arms.len() != 2 {
        return;
    }

    let first_arm = &arms[0];
    let second_arm = &arms[1];

    let first_pattern = &first_arm.patterns(db).elements(db)[0];
    let second_pattern = &second_arm.patterns(db).elements(db)[0];

    // Checks if the pattern matches Option::Some(x) => x
    let found_some_ok = match first_pattern {
        Pattern::Enum(enum_pattern) => {
            let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
            match enum_name.as_str() {
                "Option::Some" => {
                    let enum_arg = enum_pattern.pattern(db);
                    match enum_arg {
                        OptionPatternEnumInnerPattern::PatternEnumInnerPattern(x) => {
                            x.pattern(db).as_syntax_node().get_text_without_trivia(db)
                                == first_arm.expression(db).as_syntax_node().get_text_without_trivia(db)
                        }
                        OptionPatternEnumInnerPattern::Empty(_) => false,
                    }
                }
                _ => false,
            }
        }
        _ => false,
    };

    // Checks if the pattern matches Option::None => core::panic_with_felt252('err')
    let found_none_err = match second_pattern {
        Pattern::Enum(enum_pattern) => {
            let enum_name = enum_pattern.path(db).as_syntax_node().get_text_without_trivia(db);
            match enum_name.as_str() {
                "Option::None" => {
                    if let Expr::FunctionCall(func_call) = second_arm.expression(db) {
                        let func_name = func_call.path(db).as_syntax_node().get_text(db);

                        func_name == "core::panic_with_felt252" || func_name == "panic_with_felt252"
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
        _ => false,
    };

    if found_some_ok && found_none_err {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_match.as_syntax_node().stable_ptr(),
            message: MANUAL_EXPECT.to_owned(),
            severity: Severity::Warning,
        });
    }
}
