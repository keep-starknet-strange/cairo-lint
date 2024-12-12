use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Expr, ExprIf, Statement};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use if_chain::if_chain;

pub const COLLAPSIBLE_IF: &str =
    "Each `if`-statement adds one level of nesting, which makes code look more complex than it really is.";
pub(super) const LINT_NAME: &str = "collapsible_if";

/// Checks for
/// ```ignore
/// if cond {
///     if second_cond {
///         ...
///     }
/// }
/// ```
/// This can be collapsed to
/// ```ignore
/// if cond && second_cond {
///     ...
/// }
/// ```
pub fn check_collapsible_if(
    db: &dyn SemanticGroup,
    expr_if: &ExprIf,
    arenas: &Arenas,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    // Checks if the lint is allowed in any upper scope
    let mut current_node = expr_if.stable_ptr.lookup(db.upcast()).as_syntax_node();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }
    let Expr::Block(ref if_block) = arenas.exprs[expr_if.if_block] else { return };

    // TODO: Check if if block can contain only 1 statement without tail
    // Case where the if block only contains a statement and no tail
    if_chain! {
        if if_block.statements.len() == 1;
        if if_block.tail.is_none();
        // If the inner statement is an expression
        if let Statement::Expr(ref inner_expr_stmt) = arenas.statements[if_block.statements[0]];
        // And this expression is an if expression
        if let Expr::If(ref inner_if_expr) = arenas.exprs[inner_expr_stmt.expr];
        then {
            // Check if any of the ifs (outter and inner) have an else block, if it's the case don't diagnostic
            if inner_if_expr.else_block.is_some() || expr_if.else_block.is_some() {
                return;
            }

            diagnostics.push(PluginDiagnostic {
                stable_ptr: expr_if.stable_ptr.untyped(),
                message: COLLAPSIBLE_IF.to_string(),
                severity: Severity::Warning,
            });
            return;
        }
    }

    // Case where the outter if only has a tail.
    if if_block.tail.is_some_and(|tail| {
        // Check that the tail expression is a if
        let Expr::If(ref inner_if_expr) = arenas.exprs[tail] else {
            return false;
        };
        // Check if any of the ifs (outter and inner) have an else block, if it's the case don't diagnostic
        expr_if.else_block.is_none() && inner_if_expr.else_block.is_none()
    }) && if_block.statements.is_empty()
    {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_if.stable_ptr.untyped(),
            message: COLLAPSIBLE_IF.to_string(),
            severity: Severity::Warning,
        });
    }
}
