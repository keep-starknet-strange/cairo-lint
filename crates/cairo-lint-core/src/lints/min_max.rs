use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::{db::SyntaxGroup, kind::SyntaxKind, SyntaxNode};

pub const MIN_MAX: &str = "Wrong usage of min and max functions.";

pub fn check_min_max(db: &dyn SyntaxGroup, root: SyntaxNode, diagnostics: &mut Vec<PluginDiagnostic>) {
    for node in root.descendants(db) {
        if let Some(call_expr) = is_function_call(db, &node) {
            if is_min_max(db, call_expr) {
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: node.stable_ptr(),
                    message: MIN_MAX.to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }
}

fn is_function_call(db: &dyn SyntaxGroup, node: &SyntaxNode) -> Option<SyntaxNode> {
    if node.kind(db) == SyntaxKind::ExprFunctionCall {
        Some(node.clone())
    } else {
        None
    }
}

fn is_min_max(db: &dyn SyntaxGroup, call_expr: SyntaxNode) -> bool {
    let children = db.get_children(call_expr.clone());

    if let Some(func_name_node) = children.first() {
        if let Some(func_name) = func_name_node.text(db) {
            if func_name == "min" || func_name == "max" {
                if let [arg1, arg2] = &children[1..] {
                    let arg1_text = arg1.get_text(db);
                    let arg2_text = arg2.get_text(db);

                    if func_name == "min" && arg1_text > arg2_text {
                        return true;
                    } else if func_name == "max" && arg1_text < arg2_text {
                        return true;
                    }
                }
            }
        }
    }

    false
}
