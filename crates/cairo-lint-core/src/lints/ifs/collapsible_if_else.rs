use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BlockOrIf, ElseClause, Expr, ExprBlock, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const COLLAPSIBLE_IF_ELSE: &str = "Consider using else if instead of else { if ... }";

pub fn is_only_statement_if(db: &dyn SyntaxGroup, block_expr: &ExprBlock) -> bool {
    let statements = block_expr.statements(db).elements(db);
    if statements.len() != 1 {
        return false;
    }
    if let Statement::Expr(statement_expr) = &statements[0]
        && matches!(statement_expr.expr(db), Expr::If(_))
    {
        true
    } else {
        false
    }
}

pub fn check_collapsible_if_else(
    db: &dyn SyntaxGroup,
    else_clause: &ElseClause,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    if let Some(node) = else_clause.as_syntax_node().parent()
        && let Some(grand_parent) = node.parent()
        && grand_parent.has_attr_with_arg(db, "allow", "collapsible_if_else")
    {
        return;
    }
    // Extract the expression from the ElseClause
    let else_expr = else_clause.else_block_or_if(db);

    // Check if the expression is a block (not else if)
    if let BlockOrIf::Block(block_expr) = else_expr {
        let is_if = is_only_statement_if(db, &block_expr);

        if is_if {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: else_clause.stable_ptr().untyped(),
                message: COLLAPSIBLE_IF_ELSE.to_string(),
                severity: Severity::Warning,
            });
        }
    }
}
