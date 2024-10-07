use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::ExprInlineMacro;
use cairo_lang_syntax::node::helpers::WrappedArgListHelper;
use cairo_lang_syntax::node::{SyntaxNode, TypedSyntaxNode};
use cairo_lang_syntax::node::db::SyntaxGroup;
use std::str::FromStr;

// Mensajes constantes de diagnóstico
pub const ASSERT_MSG: &str = "assert!(true) is redundant and can be removed.";
pub const ASSERT_MSG_FALSE: &str = "assert!(false) can be replaced with panic!.";

fn evaluate_expression(expr: &str) -> Option<bool> {
    bool::from_str(expr).ok()
}

pub fn check_assert(db: &dyn SyntaxGroup, node: SyntaxNode, diagnostics: &mut Vec<PluginDiagnostic>) {
    // Crear una instancia de ExprInlineMacro a partir del nodo de sintaxis
    let expr_macro = ExprInlineMacro::from_syntax_node(db, node.clone());

    // Extraer la ruta (el nombre de la macro, por ejemplo "assert") y los argumentos
    let path = expr_macro.path(db);
    let arguments = expr_macro.arguments(db);

    // Verificar si el nombre de la macro es "assert"
    let path_text = path.as_syntax_node().get_text_without_trivia(db);
    if path_text == "assert" {
        // Si la macro tiene argumentos, proceder
        if let Some(arg_list) = arguments.arg_list(db) {
            let args = arg_list.elements(db);

            // Asegurarse de que la macro tenga exactamente un argumento
            if args.len() == 1 {
                if let Some(first_arg) = args.get(0) {
                    let arg_clause = first_arg.arg_clause(db);
                    let arg_text = arg_clause.as_syntax_node().get_text_without_trivia(db);

                    // Intentar evaluar el argumento como un valor booleano
                    if let Some(result) = evaluate_expression(&arg_text) {
                        if result {
                            // Si la expresión es true, generar una advertencia
                            diagnostics.push(PluginDiagnostic {
                                stable_ptr: node.stable_ptr(),
                                message: ASSERT_MSG.to_string(),
                                severity: Severity::Warning,
                            });
                        } else {
                            // Si la expresión es false, generar un error
                            diagnostics.push(PluginDiagnostic {
                                stable_ptr: node.stable_ptr(),
                                message: ASSERT_MSG_FALSE.to_string(),
                                severity: Severity::Error,
                            });
                        }
                    } 
                }
            }
        }
    }
}
