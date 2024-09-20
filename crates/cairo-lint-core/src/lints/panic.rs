use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::ExprFunctionCall;

pub const PANIC_IN_CODE: &str = "Using `panic!` is discouraged.";
const PANIC: &str = "\"panic\"";

pub fn check_panic_usage(
    db: &dyn SemanticGroup,
    expr_function_call: &ExprFunctionCall,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {

    let function_id = expr_function_call.function;

    if function_id.name(db) == PANIC {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_function_call.stable_ptr.into(),
            message: PANIC_IN_CODE.to_owned(),
            severity: Severity::Warning,
        });
    }   

}

