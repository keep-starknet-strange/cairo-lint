use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::SyntaxNode;
use cairo_lang_syntax::node::db::SyntaxGroup;
use std::str::FromStr;

pub const ASSERT_MSG: &str = "assert!(true) is redundant and can be removed.";
pub const ASSERT_MSG_FALSE: &str = "assert!(false) can be replaced with panic!.";


fn evaluate_expression(expr: &str) -> Option<bool> {
    bool::from_str(expr).ok()
}

pub fn check_assert(db: &dyn SyntaxGroup, node: SyntaxNode, diagnostics: &mut Vec<PluginDiagnostic>) {
    let cloned_node = node.clone();
    let text = cloned_node.get_text_without_trivia(db);

    //ExprInlineMacro::from_syntax_node(node)
    if text.starts_with("assert!(") {
        let inner_expr = text
            .trim_start_matches("assert!(")
            .trim_end_matches(|c| c == ')' || c == ';')
            .trim();


       
        if let Some(result) = evaluate_expression(inner_expr) {
            if result {
                
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: node.stable_ptr(),
                    message: ASSERT_MSG.to_string(),
                    severity: Severity::Warning,
                });
            } else {
             
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: node.stable_ptr(),
                    message: ASSERT_MSG_FALSE.to_string(),
                    severity: Severity::Error,
                });
            }
        } else {
            println!("La expresión no es una constante booleana evaluable.");
        }
    }
}
