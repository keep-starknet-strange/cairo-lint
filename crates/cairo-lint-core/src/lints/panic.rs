use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, ExprFunctionCall};
use cairo_lang_semantic::FunctionId;

pub const PANIC_IN_CODE: &str = "Using `panic!` is discouraged.";

pub fn check_panic_usage(
    db: &dyn SemanticGroup,
    expr_function_call: &ExprFunctionCall,
    diagnostics: &mut Vec<PluginDiagnostic>,
    _arenas: &Arenas,  
) {
    println!("Lint `check_panic_usage` ejecutado");
    // Obtiene el identificador de la función
    let function_id: FunctionId = expr_function_call.function;
    
    // Obtiene el nombre de la función desde el `FunctionId`
    let function_name = function_id.name(db);

    println!("Checking function: {}", function_name);
    
    // Verifica si el nombre de la función es "panic"
    if function_name == "panic!" {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_function_call.stable_ptr.into(),
            message: PANIC_IN_CODE.to_owned(),
            severity: Severity::Warning,
        });
        println!("Detected panic! in the code.");
    }
}