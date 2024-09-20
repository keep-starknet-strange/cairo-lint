//Check if the if and else blocks are the same.
//This is a common mistake where the else block is the same as the if block.
//This is likely a copy & paste error.
//
//Lints:
// - if_else_same
//
//Quick-fixes:
// - if_else_same_quickfix

use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BlockOrIf, ElseClause, ExprBlock, ExprIf};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedSyntaxNode, SyntaxNode};

pub const IF_ELSE_SAME: &str = "if and else blocks are the same. This is likely a copy & paste error.";

pub fn check_if_else_same(db: &dyn SyntaxGroup, expr: &ExprIf, diagnostics: &mut Vec<PluginDiagnostic>) {
    let if_block = expr.if_block(db);
    let else_clause = expr.else_clause(db);

    if let BlockOrIf::Block(else_block) = else_clause {
        let else_block_text = else_block.as_syntax_node().get_text(db);
        let if_block_text = if_block.as_syntax_node().get_text(db);

        if else_block_text == if_block_text {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: expr.as_syntax_node().stable_ptr(),
                message: IF_ELSE_SAME.to_string(),
                severity: Severity::Warning,
            });
        }
    }
}