use cairo_lang_defs::diagnostic_utils::StableLocation;
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_filesystem::db::get_originating_location;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::ExprFunctionCall;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use if_chain::if_chain;

pub const PANIC_IN_CODE: &str = "Leaving `panic` in the code is discouraged.";
const PANIC: &str = "core::panics::panic";
pub const ALLOWED: [&str; 1] = [LINT_NAME];
const LINT_NAME: &str = "panic";

/// Checks for panic usage.
pub fn check_panic_usage(
    db: &dyn SemanticGroup,
    expr_function_call: &ExprFunctionCall,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    // Checks if the lint is allowed in an upper scope
    let mut current_node = expr_function_call
        .stable_ptr
        .lookup(db.upcast())
        .as_syntax_node();
    let init_node = current_node.clone();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }

    // If the function is not the panic function from the corelib return
    if expr_function_call.function.full_name(db) != PANIC {
        return;
    }

    // Get the origination location of this panic as there is a `panic!` macro that gerates virtual
    // files
    let initial_file_id =
        StableLocation::new(expr_function_call.stable_ptr.untyped()).file_id(db.upcast());
    let (file_id, span) = get_originating_location(
        db.upcast(),
        initial_file_id,
        expr_function_call
            .stable_ptr
            .lookup(db.upcast())
            .as_syntax_node()
            .span(db.upcast()),
    );
    // If the panic comes from a real file (macros generate code in new virtual files)
    if initial_file_id == file_id {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: init_node.stable_ptr(),
            message: PANIC_IN_CODE.to_owned(),
            severity: Severity::Warning,
        });
    } else {
        // If the originating location is a different file get the syntax node that generated the
        // code that contains a panic.
        if_chain! {
            if let Some(text_position) = span.position_in_file(db.upcast(), file_id);
            if let Ok(file_node) = db.file_syntax(file_id);
            then {
                let syntax_node = file_node.lookup_position(db.upcast(), text_position.start);
                // Checks if the lint is allowed in the original file
                let mut current_node = syntax_node.clone();
                while let Some(node) = current_node.parent() {
                    if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
                        return;
                    }
                    current_node = node;
                }
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: syntax_node.stable_ptr(),
                    message: PANIC_IN_CODE.to_owned(),
                    severity: Severity::Warning,
                });
            }
        }
    }
}
