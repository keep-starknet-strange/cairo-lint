use cairo_lang_semantic::{Arenas, Expr, ExprFunctionCall, ExprFunctionCallArg};

pub mod collapsible_if;
pub mod collapsible_if_else;
pub mod equatable_if_let;
pub mod ifs_same_cond;

pub const ALLOWED: [&str; 4] =
    [collapsible_if_else::LINT_NAME, equatable_if_let::LINT_NAME, ifs_same_cond::LINT_NAME, collapsible_if::LINT_NAME];

fn ensure_no_ref_arg(arenas: &Arenas, func_call: &ExprFunctionCall) -> bool {
    func_call.args.iter().any(|arg| match arg {
        ExprFunctionCallArg::Reference(_) => true,
        ExprFunctionCallArg::Value(expr_id) => match &arenas.exprs[*expr_id] {
            Expr::FunctionCall(expr_func) => ensure_no_ref_arg(arenas, expr_func),
            _ => false,
        },
    })
}
