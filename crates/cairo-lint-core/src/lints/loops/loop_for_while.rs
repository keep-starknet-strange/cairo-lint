use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Expr, ExprId, ExprLoop, Statement};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use if_chain::if_chain;

pub const LOOP_FOR_WHILE: &str = "you seem to be trying to use `loop`. Consider replacing this `loop` with a `while` \
                                  loop for clarity and conciseness";

pub const ALLOWED: [&str; 1] = [LINT_NAME];
pub(super) const LINT_NAME: &str = "loop_for_while";

/// Checks for
/// ```ignore
/// loop {
///     ...
///     if cond {
///         break;
///     }
/// }
/// ```
/// Which can be rewritten as a while loop
/// ```ignore
/// while cond {
///     ...
/// }
/// ```
pub fn check_loop_for_while(
    db: &dyn SemanticGroup,
    loop_expr: &ExprLoop,
    arenas: &Arenas,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    // Check if the lint is allowed in an upper scope
    let mut current_node = loop_expr.stable_ptr.lookup(db.upcast()).as_syntax_node();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }

    // Get the else block  expression
    let Expr::Block(block_expr) = &arenas.exprs[loop_expr.body] else {
        return;
    };
    // Checks if one of the statements is an if expression that only contains a break instruction
    for statement in &block_expr.statements {
        if_chain! {
            if let Statement::Expr(ref expr_statement) = arenas.statements[*statement];
            if check_if_contains_break(&expr_statement.expr, arenas);
            then {
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: loop_expr.stable_ptr.untyped(),
                    message: LOOP_FOR_WHILE.to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    // Do the same thing if the if is in the tail of the block
    if_chain! {
        if let Some(tail_expr) = block_expr.tail;
        if check_if_contains_break(&tail_expr, arenas);
        then {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: loop_expr.stable_ptr.untyped(),
                message: LOOP_FOR_WHILE.to_string(),
                severity: Severity::Warning,
            });
        }
    }
}

fn check_if_contains_break(expr: &ExprId, arenas: &Arenas) -> bool {
    if_chain! {
        // Is an if expression
        if let Expr::If(ref if_expr) = arenas.exprs[*expr];
        // Get the block
        if let Expr::Block(ref if_block) = arenas.exprs[if_expr.if_block];
        // Get the first statement of the if
        if let Some(inner_stmt) = if_block.statements.first();
        // Is it a break statement
        if matches!(arenas.statements[*inner_stmt], Statement::Break(_));
        then {
            return true;
        }
    }
    false
}
