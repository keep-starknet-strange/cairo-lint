use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Condition, Expr, ExprFunctionCall, ExprFunctionCallArg, ExprIf, LogicalOperator};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use num_bigint::BigInt;

pub const IMPOSSIBLE_COMPARISON: &str = "Impossible condition, replace by false";

pub const ALLOWED: [&str; 1] = [LINT_NAME];

const LINT_NAME: &str = "impossible_comparison";

pub fn check_impossible_comparision(
    db: &dyn SemanticGroup,
    arenas: &Arenas,
    expr_if: &ExprIf,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    // Checks if the lint is allowed in an upper scope
    let mut current_node = expr_if.stable_ptr.lookup(db.upcast()).as_syntax_node();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }

    let cond_expr = match &expr_if.condition {
        Condition::BoolExpr(expr_id) => &arenas.exprs[*expr_id],
        Condition::Let(_, _) => {
            return;
        }
    };

    let Expr::LogicalOperator(logic_expr) = cond_expr else {
        return;
    };

    if logic_expr.op != LogicalOperator::AndAnd {
        return;
    }

    let Expr::FunctionCall(lhs_func) = &arenas.exprs[logic_expr.lhs] else {
        return;
    };
    let Expr::FunctionCall(rhs_func) = &arenas.exprs[logic_expr.rhs] else {
        return;
    };

    let (lhs_var_name, lhs_op, lhs_int) = match get_values(db, lhs_func, arenas) {
        Some(vals) => vals,
        None => {
            return;
        }
    };
    let (rhs_var_name, rhs_op, rhs_int) = match get_values(db, rhs_func, arenas) {
        Some(vals) => vals,
        None => {
            return;
        }
    };

    if lhs_var_name != rhs_var_name {
        return;
    }

    if !match (lhs_op, rhs_op) {
        ("gt", "lt") => lhs_int >= rhs_int,
        ("gt", "le") => lhs_int >= rhs_int,
        ("ge", "lt") => lhs_int >= rhs_int,
        ("ge", "le") => lhs_int > rhs_int,
        ("lt", "gt") => lhs_int <= rhs_int,
        ("lt", "ge") => lhs_int <= rhs_int,
        ("le", "gt") => lhs_int <= rhs_int,
        ("le", "ge") => lhs_int < rhs_int,
        _ => true,
    } {
        return;
    }

    diagnostics.push(PluginDiagnostic {
        stable_ptr: expr_if.stable_ptr.untyped(),
        message: IMPOSSIBLE_COMPARISON.to_string(),
        severity: Severity::Warning,
    });
}

fn get_values(
    db: &dyn SemanticGroup,
    func_call: &ExprFunctionCall,
    arenas: &Arenas,
) -> Option<(String, &'static str, BigInt)> {
    // Check is >= or > or < or <=
    let full_name = func_call.function.full_name(db);
    let logic_op = if full_name.contains("core::integer::") {
        if full_name.contains("PartialOrd::gt") {
            "gt"
        } else if full_name.contains("PartialOrd::ge") {
            "ge"
        } else if full_name.contains("PartialOrd::lt") {
            "lt"
        } else if full_name.contains("PartialOrd::le") {
            "le"
        } else {
            return None;
        }
    } else {
        return None;
    };

    // Check lhs is var
    let ExprFunctionCallArg::Value(v) = &func_call.args[0] else {
        return None;
    };
    let Expr::Var(expr_var) = arenas.exprs[*v].clone() else {
        return None;
    };

    // Check rhs is int
    let ExprFunctionCallArg::Value(v) = &func_call.args[1] else {
        return None;
    };
    let Expr::Literal(ref litteral_expr) = arenas.exprs[*v].clone() else {
        return None;
    };

    Some((
        expr_var.stable_ptr.lookup(db.upcast()).as_syntax_node().get_text(db.upcast()),
        logic_op,
        litteral_expr.value.clone(),
    ))
}
