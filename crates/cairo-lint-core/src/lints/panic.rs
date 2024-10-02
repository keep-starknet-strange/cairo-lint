use cairo_lang_defs::diagnostic_utils::StableLocation;
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_filesystem::db::get_originating_location;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::ExprFunctionCall;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::kind::SyntaxKind;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const PANIC_IN_CODE: &str = "Leaving `panic!` in the code is discouraged.";
const PANIC: &str = "\"panic\"";
pub const ALLOWED: [&str; 1] = [LINT_NAME];
const LINT_NAME: &str = "panic";

pub fn check_panic_usage(
    db: &dyn SemanticGroup,
    expr_function_call: &ExprFunctionCall,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let function_id = expr_function_call.function;

    if function_id.name(db) == PANIC {
        let initial_file_id = StableLocation::new(expr_function_call.stable_ptr.untyped()).file_id(db.upcast());
        let (file_id, span) = get_originating_location(
            db.upcast(),
            initial_file_id,
            expr_function_call.stable_ptr.lookup(db.upcast()).as_syntax_node().span(db.upcast()),
        );
        if initial_file_id == file_id {
            let node = expr_function_call.stable_ptr.lookup(db.upcast()).as_syntax_node();
            if let Some(node) = node.parent()
                && node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME)
            {
                return;
            }
            diagnostics.push(PluginDiagnostic {
                stable_ptr: node.stable_ptr(),
                message: PANIC_IN_CODE.to_owned(),
                severity: Severity::Warning,
            });
        } else if let Some(text_position) = span.position_in_file(db.upcast(), file_id)
            && let Ok(file_node) = db.file_syntax(file_id)
        {
            let syntax_node = file_node.lookup_position(db.upcast(), text_position.start);
            let mut curr_node = syntax_node.clone();
            while let Some(node) = curr_node.parent()
                && (curr_node.kind(db.upcast()) != SyntaxKind::ExprInlineMacro
                    || curr_node.kind(db.upcast()) == SyntaxKind::ExprFunctionCall)
            {
                curr_node = node;
            }
            if let Some(node) = curr_node.parent()
                && node.has_attr_with_arg(db.upcast(), "allow", "panic")
            {
                return;
            }
            diagnostics.push(PluginDiagnostic {
                stable_ptr: syntax_node.stable_ptr(),
                message: PANIC_IN_CODE.to_owned(),
                severity: Severity::Warning,
            });
        }
    }
}
