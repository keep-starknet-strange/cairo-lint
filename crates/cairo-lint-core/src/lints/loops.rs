use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Expr, ExprLoop, Statement};

pub const LOOP_MATCH_POP_FRONT: &str =
    "you seem to be trying to use `loop` for iterating over a span. Consider using `for in`";

const SPAN_MATCH_POP_FRONT: &str = "\"SpanImpl::pop_front\"";

pub fn check_loop_match_pop_front(
    db: &dyn SemanticGroup,
    loop_expr: &ExprLoop,
    diagnostics: &mut Vec<PluginDiagnostic>,
    arenas: &Arenas,
) {
    let Expr::Block(expr_block) = &arenas.exprs[loop_expr.body] else {
        return;
    };
    if let Some(tail) = &expr_block.tail
        && let Expr::Match(expr_match) = &arenas.exprs[*tail]
        && let Expr::FunctionCall(func_call) = &arenas.exprs[expr_match.matched_expr]
        && func_call.function.name(db) == SPAN_MATCH_POP_FRONT
    {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: loop_expr.stable_ptr.into(),
            message: LOOP_MATCH_POP_FRONT.to_owned(),
            severity: Severity::Warning,
        });
        return;
    }
    for statement in &expr_block.statements {
        if let Statement::Expr(stmt_expr) = &arenas.statements[*statement]
            && let Expr::Match(expr_match) = &arenas.exprs[stmt_expr.expr]
        {
            let Expr::FunctionCall(func_call) = &arenas.exprs[expr_match.matched_expr] else {
                continue;
            };
            if func_call.function.name(db) == SPAN_MATCH_POP_FRONT {
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: loop_expr.stable_ptr.into(),
                    message: LOOP_MATCH_POP_FRONT.to_owned(),
                    severity: Severity::Warning,
                })
            }
        }
    }
}
