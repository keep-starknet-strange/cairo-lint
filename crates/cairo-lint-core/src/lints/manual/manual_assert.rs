use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{Condition, Expr, ExprBlock, ExprIf, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const MANUAL_ASSERT: &str = "`assert!` is simpler than `if`-then-`panic!`";

pub fn contains_panic(db: &dyn SyntaxGroup, block_expr: &ExprBlock) -> bool {
    let statements = block_expr.statements(db).elements(db);
    if let Statement::Expr(statement_expr) = &statements[0] {
        if let Expr::InlineMacro(inline_macro) = &statement_expr.expr(db) {
            let macro_name: String = inline_macro.path(db).node.clone().get_text_without_trivia(db);
            if macro_name == "panic" {
                return true;
            }
        }
    }
    false
}

pub fn check_if_then_panic(db: &dyn SyntaxGroup, expr: &ExprIf, diagnostics: &mut Vec<PluginDiagnostic>) {
    let condition = expr.condition(db);
    let block_expr = expr.if_block(db);
    if let Condition::Expr(_condition_expr) = condition
        && contains_panic(db, &block_expr)
    {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr.stable_ptr().untyped(),
            message: MANUAL_ASSERT.to_string(),
            severity: Severity::Warning,
        });
    }
}
