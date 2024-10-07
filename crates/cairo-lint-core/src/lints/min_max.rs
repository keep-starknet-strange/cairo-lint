use cairo_lang_defs::diagnostic_utils::StableLocation;
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::{db::SemanticGroup, ExprFunctionCallArg, ExprFunctionCall};
use cairo_lang_filesystem::db::get_originating_location;

pub const MIN_MAX_WARNING: &str = "Wrong usage of min and max functions.";
const MIN: &str = "\"min\"";
const MAX: &str = "\"max\"";

pub fn check_min_max(
    db: &dyn SemanticGroup,
    expr_function_call: &ExprFunctionCall,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {

    let function_id = expr_function_call.function;

    if is_min_max_function(db, &expr_function_call) {
        let (file_id, span) = get_originating_location(
            db.upcast(),
            StableLocation::new(expr_function_call.stable_ptr.untyped()).file_id(db.upcast()),
            expr_function_call.stable_ptr.lookup(db.upcast()).as_syntax_node().span(db.upcast()),
        );

        if let Some(text_position) = span.position_in_file(db.upcast(), file_id) {
            if let Ok(syntax_node) = db.file_syntax(file_id) {
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: syntax_node.lookup_position(db.upcast(), text_position.start).stable_ptr(),
                    message: MIN_MAX_WARNING.to_owned(),
                    severity: Severity::Warning,
                });
            }
        }
    }
}

fn is_min_max_function(db: &dyn SemanticGroup, call_expr: &ExprFunctionCall) -> bool {
    let function_id = call_expr.function;
    let func_name = function_id.name(db);

    if func_name == MIN || func_name == MAX {
        if call_expr.args.len() == 2 {
            let arg1 = &call_expr.args[0];
            let arg2 = &call_expr.args[1];

            match func_name.as_str() {
                "min" => {
                    if let (ExprFunctionCallArg::Value(val1), ExprFunctionCallArg::Value(val2)) = (arg1, arg2) {
                        return val1 < val2;
                    }
                },
                "max" => {
                    if let (ExprFunctionCallArg::Value(val1), ExprFunctionCallArg::Value(val2)) = (arg1, arg2) {
                        return val2 < val1;
                    }
                },
                _ => return false,
            }
        }
    }
    false
}
