use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Expr, ExprBlock, ExprIf, Statement};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use if_chain::if_chain;

pub const COLLAPSIBLE_IF_ELSE: &str = "Consider using else if instead of else { if ... }";
pub(super) const LINT_NAME: &str = "collapsible_if_else";

/// Checks for
/// ```ignore
/// if cond {
///     ...
/// } else {
///     if second_cond {
///         ...
///     }
/// }
/// ```
/// This can be collapsed to:
/// ```ignore
/// if cond {
///     ...
/// } else if second_cond {
///     ...
/// }
/// ```
pub fn check_collapsible_if_else(
    db: &dyn SemanticGroup,
    expr_if: &ExprIf,
    arenas: &Arenas,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let mut current_node = expr_if.stable_ptr.lookup(db.upcast()).as_syntax_node();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }
    // Extract the expression from the ElseClause
    let Some(else_block) = expr_if.else_block else {
        return;
    };

    let Expr::Block(block_expr) = &arenas.exprs[else_block] else {
        return;
    };
    // Check if the expression is a block (not else if)
    let is_if = is_only_statement_if(block_expr, arenas);

    if is_if {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_if.stable_ptr.untyped(),
            message: COLLAPSIBLE_IF_ELSE.to_string(),
            severity: Severity::Warning,
        });
    }
}

fn is_only_statement_if(block_expr: &ExprBlock, arenas: &Arenas) -> bool {
    if block_expr.statements.len() == 1 && block_expr.tail.is_none() {
        if_chain! {
          if let Statement::Expr(statement_expr) = &arenas.statements[block_expr.statements[0]];
          if matches!(arenas.exprs[statement_expr.expr], Expr::If(_));
          then {
            return true;
          } else {
            return false;
          }
        }
    }

    if_chain! {
      if let Some(tail) = block_expr.tail;
      if block_expr.statements.is_empty();
      then {
        return matches!(arenas.exprs[tail], Expr::If(_));
      }
    }

    false
}
