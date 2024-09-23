use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{Expr, PathSegment};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedSyntaxNode, Terminal};
use cairo_lang_syntax::node::helpers::PathSegmentEx;
use cairo_lang_semantic::resolve::AsSegments;


/// Diagnostic message for usage of `unwrap()`
pub const UNWRAP_USED: &str = "Use of unwrap() detected. Consider using '?' or 'expect()' instead.";

/// Function to check for the usage of `unwrap()` in expressions.
pub fn check_unwrap_used(
    ctx: &dyn SyntaxGroup,
    expr: &Expr,
) -> Option<PluginDiagnostic> {
    // Check if the expression is a function call.
    if let Expr::FunctionCall(func_call) = expr {
        // Get the path segment of the called function.
        if let Some(last_segment) = func_call.path(ctx).to_segments(ctx).last() {
            // Check if the last path segment is the unwrap function.
            if is_unwrap_function(ctx, last_segment) {
                return Some(PluginDiagnostic {
                    stable_ptr: expr.stable_ptr().into(),
                    message: UNWRAP_USED.to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }
    None
}

/// Helper function to determine if the called function is `unwrap()`.
fn is_unwrap_function(ctx: &dyn SyntaxGroup, segment: &PathSegment) -> bool {
    segment.identifier_ast(ctx).text(ctx) == "unwrap"
}