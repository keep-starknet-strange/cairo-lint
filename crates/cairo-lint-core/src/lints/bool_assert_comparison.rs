use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::SyntaxNode;
use cairo_lang_syntax::node::db::SyntaxGroup;
use std::str::FromStr;

pub const ASSERT_EQ_FALSE_MSG: &str = "assert_eq!(expr, false) can be replaced with assert!(!expr).";
pub const ASSERT_EQ_TRUE_MSG: &str = "assert_eq!(expr, true) can be replaced with assert!(expr).";
pub const ASSERT_NE_FALSE_MSG: &str = "assert_ne!(expr, false) can be replaced with assert!(expr).";
pub const ASSERT_NE_TRUE_MSG: &str = "assert_ne!(expr, true) can be replaced with assert!(!expr).";

fn evaluate_expression(expr: &str) -> Option<bool> {
    bool::from_str(expr).ok()
}

pub fn check_assert(db: &dyn SyntaxGroup, node: SyntaxNode, diagnostics: &mut Vec<PluginDiagnostic>) {
    let cloned_node = node.clone();
    let text = cloned_node.get_text_without_trivia(db);

    if text.starts_with("assert_eq!(") {
        let inner_expr = text
            .trim_start_matches("assert_eq!(")
            .trim_end_matches(')')
            .trim_end_matches(';')
            .trim();

        let parts: Vec<&str> = inner_expr.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let right = parts[1];

            if let Some(right_val) = evaluate_expression(right) {
                if right_val {
                    diagnostics.push(PluginDiagnostic {
                        stable_ptr: node.stable_ptr(),
                        message: ASSERT_EQ_TRUE_MSG.to_string(),
                        severity: Severity::Warning,
                    });
                } else {
                    diagnostics.push(PluginDiagnostic {
                        stable_ptr: node.stable_ptr(),
                        message: ASSERT_EQ_FALSE_MSG.to_string(),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    } else if text.starts_with("assert_ne!(") {
        let inner_expr = text
            .trim_start_matches("assert_ne!(")
            .trim_end_matches(')')
            .trim_end_matches(';')
            .trim();

        let parts: Vec<&str> = inner_expr.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let right = parts[1];

            if let Some(right_val) = evaluate_expression(right) {
                if right_val {
                    diagnostics.push(PluginDiagnostic {
                        stable_ptr: node.stable_ptr(),
                        message: ASSERT_NE_TRUE_MSG.to_string(),
                        severity: Severity::Warning,
                    });
                } else {
                    diagnostics.push(PluginDiagnostic {
                        stable_ptr: node.stable_ptr(),
                        message: ASSERT_NE_FALSE_MSG.to_string(),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }
}
