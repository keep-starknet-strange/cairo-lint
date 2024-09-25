use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::SyntaxNode;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::ast::ExprInlineMacro; 
use cairo_lang_syntax::node::TypedSyntaxNode;
use cairo_lang_syntax::node::helpers::WrappedArgListHelper;
use std::str::FromStr;

pub const ASSERT_EQ_FALSE_MSG: &str = "assert_eq!(expr, false) can be replaced with assert!(!expr).";
pub const ASSERT_EQ_TRUE_MSG: &str = "assert_eq!(expr, true) can be replaced with assert!(expr).";
pub const ASSERT_NE_FALSE_MSG: &str = "assert_ne!(expr, false) can be replaced with assert!(expr).";
pub const ASSERT_NE_TRUE_MSG: &str = "assert_ne!(expr, true) can be replaced with assert!(!expr).";

fn evaluate_expression(expr: &str) -> Option<bool> {
    bool::from_str(expr).ok()
}

pub fn check_assert(db: &dyn SyntaxGroup, node: SyntaxNode, diagnostics: &mut Vec<PluginDiagnostic>) {
    let expr_macro: ExprInlineMacro = ExprInlineMacro::from_syntax_node(db, node.clone());
    let path = expr_macro.path(db);
    let arguments = expr_macro.arguments(db);

    if let Some(arg_list) = arguments.arg_list(db) {
        let args = arg_list.elements(db);
        if args.len() == 2 {
            if let Some(right_arg) = args.get(1) {
                let arg_clause = right_arg.arg_clause(db);
                let right_text = arg_clause.as_syntax_node().get_text_without_trivia(db);
                if let Some(right_val) = evaluate_expression(&right_text) {
                    let (message, _is_eq) = if path.as_syntax_node().get_text_without_trivia(db).contains("assert_eq") {
                        (if right_val { ASSERT_EQ_TRUE_MSG } else { ASSERT_EQ_FALSE_MSG }, true)
                    } else {
                        (if right_val { ASSERT_NE_TRUE_MSG } else { ASSERT_NE_FALSE_MSG }, false)
                    };

                    diagnostics.push(PluginDiagnostic {
                        stable_ptr: node.stable_ptr(),
                        message: message.to_string(),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }
}
