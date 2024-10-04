use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Expr, ExprId, ExprLoop, Statement};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const LOOP_FOR_WHILE: &str = "you seem to be trying to use `loop`. Consider replacing this `loop` with a `while` \
                                  loop for clarity and conciseness";

pub const ALLOWED: [&str; 1] = [LINT_NAME];
const LINT_NAME: &str = "loop_for_while";

pub fn check_loop_for_while(
    db: &dyn SemanticGroup,
    loop_expr: &ExprLoop,
    arenas: &Arenas,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let mut current_node = loop_expr.stable_ptr.lookup(db.upcast()).as_syntax_node();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }

    let Expr::Block(block_expr) = &arenas.exprs[loop_expr.body] else {
        return;
    };
    for statement in &block_expr.statements {
        if let Statement::Expr(ref expr_statement) = arenas.statements[*statement]
            && check_if_contains_break(&expr_statement.expr, arenas)
        {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: loop_expr.stable_ptr.untyped(),
                message: LOOP_FOR_WHILE.to_string(),
                severity: Severity::Warning,
            });
        }
    }
    if let Some(tail_expr) = block_expr.tail
        && check_if_contains_break(&tail_expr, arenas)
    {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: loop_expr.stable_ptr.untyped(),
            message: LOOP_FOR_WHILE.to_string(),
            severity: Severity::Warning,
        });
    }
}

fn check_if_contains_break(expr: &ExprId, arenas: &Arenas) -> bool {
    if let Expr::If(ref if_expr) = arenas.exprs[*expr] {
        let Expr::Block(ref if_block) = arenas.exprs[if_expr.if_block] else {
            return false;
        };

        if_block
            .statements
            .iter()
            .any(|inner_statement| matches!(arenas.statements[*inner_statement], Statement::Break(_)))
    } else {
        false
    }
}
