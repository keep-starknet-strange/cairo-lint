use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{Expr, ExprFunctionCall};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::helpers::GetIdentifier;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const CLONE_ON_COPY: &str = "Copy types implement Clone for generics, not for using the clone method on a concrete type.";

pub fn check_clone_on_copy(db: &dyn SyntaxGroup, expr: &Expr, diagnostics: &mut Vec<PluginDiagnostic>) {
    if let Expr::FunctionCall(func_call) = expr {
        if is_clone_call(db, func_call) && is_copy_type(db, func_call) {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: expr.stable_ptr().untyped(),
                message: CLONE_ON_COPY.to_string(),
                severity: Severity::Warning,
            });
        }
    }
}

fn is_clone_call(db: &dyn SyntaxGroup, func_call: &ExprFunctionCall) -> bool {
    let path = func_call.path(db);
    let segments = path.elements(db);
    if let Some(last_segment) = segments.last() {
        let identifier = last_segment.identifier(db);
        return identifier.as_str() == "clone";
    }
    false
}

fn is_copy_type(db: &dyn SyntaxGroup, func_call: &ExprFunctionCall) -> bool {
    // This function needs to be implemented based on Cairo's Copy types according to the Cairo Book
    // For now, assume that certain primitive types are Copy
    let path = func_call.path(db);
    let segments = path.elements(db);
    if segments.len() > 1 {
        let second_last_segment = &segments[segments.len() - 2];
        let type_name = second_last_segment.identifier(db);
        return matches!(type_name.as_str(), "u8" | "u16" | "u32" | "u64" | "usize" | "felt252");
    }
    false
}