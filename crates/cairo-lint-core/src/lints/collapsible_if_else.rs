use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{Expr, ElseClause, BlockOrIf, ExprBlock, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const COLLAPSIBLE_IF_ELSE: &str = "Consider using 'else if' instead of 'else { if ... }'";

pub fn is_first_statement_if(db: &dyn SyntaxGroup, block_expr: &ExprBlock) -> bool {
    // Get the list of statements from the block expression
    let statements: Vec<Statement> = block_expr.statements(db).elements(db);

    // Check if the first statement is an `if` statement
    if let Some(first_statement) = statements.first() {

        if let Statement::Expr(statement_expr) = first_statement {
            let first_statement_expr = statement_expr.expr(db);
            if let Expr::If(_) = first_statement_expr {
                true
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

pub fn check_collapsible_if_else(db: &dyn SyntaxGroup, else_clause: &ElseClause, diagnostics: &mut Vec<PluginDiagnostic>) {

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
