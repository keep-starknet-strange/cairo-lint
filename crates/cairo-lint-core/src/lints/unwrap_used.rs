use cairo_lang_defs::diagnostic_utils::StableLocation;
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_filesystem::db::get_originating_location;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::ExprFunctionCall;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const UNWRAP_USED: &str = "Use of unwrap() detected. Consider using '?' or 'expect()' instead.";
const UNWRAP: &str = "\"unwrap\"";

pub fn check_unwrap_used(
    db: &dyn SemanticGroup,
    expr_function_call: &ExprFunctionCall,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let function_id = expr_function_call.function;
    if function_id.name(db) == UNWRAP {
        let (file_id, span) = get_originating_location(
            db.upcast(),
            StableLocation::new(expr_function_call.stable_ptr.untyped()).file_id(db.upcast()),
            expr_function_call.stable_ptr.lookup(db.upcast()).as_syntax_node().span(db.upcast()),
        );
        if let Some(text_position) = span.position_in_file(db.upcast(), file_id) {
            if let Ok(syntax_node) = db.file_syntax(file_id) {
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: syntax_node.lookup_position(db.upcast(), text_position.start).stable_ptr(),
                    message: UNWRAP_USED.to_owned(),
                    severity: Severity::Warning,
                });
            }
        }
    }
}
