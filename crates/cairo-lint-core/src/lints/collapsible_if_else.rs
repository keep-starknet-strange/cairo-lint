use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BlockOrIf, ElseClause, Expr, ExprBlock, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const COLLAPSIBLE_IF_ELSE: &str = "Consider using else if instead of else { if ... }";

pub fn is_first_statement_if(db: &dyn SyntaxGroup, block_expr: &ExprBlock) -> bool {
    block_expr
        .statements(db)
        .elements(db)
        .first()
        .and_then(|first_statement| {
            if let Statement::Expr(statement_expr) = first_statement { Some(statement_expr.expr(db)) } else { None }
        })
        .map_or(false, |expr| matches!(expr, Expr::If(_)))
}

pub fn check_collapsible_if_else(
    db: &dyn SyntaxGroup,
    else_clause: &ElseClause,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    // Extract the expression from the ElseClause
    let else_expr = else_clause.else_block_or_if(db);

    // Check if the expression is a block (not else if)
    if let BlockOrIf::Block(block_expr) = else_expr {
        let is_if = is_first_statement_if(db, &block_expr);

        if is_if {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: else_clause.stable_ptr().untyped(),
                message: COLLAPSIBLE_IF_ELSE.to_string(),
                severity: Severity::Warning,
            });
        }
    }
}
