use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Expr, ExprFunctionCallArg, ExprLogicalOperator, LogicalOperator};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use num_bigint::BigInt;

use super::function_trait_name_from_fn_id;
use crate::lints::{EQ, GE, GT, LE, LT};

pub const IMPOSSIBLE_COMPARISON: &str = "Impossible condition, always false";

pub const ALLOWED: [&str; 1] = [LINT_NAME];
const LINT_NAME: &str = "impossible_comparison";

pub fn check_impossible_comparision(
    db: &dyn SemanticGroup,
    expr_logical: &ExprLogicalOperator,
    arenas: &Arenas,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let mut current_node = expr_logical.stable_ptr.lookup(db.upcast()).as_syntax_node();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }

    let Expr::FunctionCall(lhs_comparison) = &arenas.exprs[expr_logical.lhs] else {
        return;
    };
    if lhs_comparison.args.len() != 2 {
        return;
    }

    let Expr::FunctionCall(rhs_comparison) = &arenas.exprs[expr_logical.rhs] else {
        return;
    };
    if rhs_comparison.args.len() != 2 {
        return;
    }

    let (lhs_fn_trait_name, rhs_fn_trait_name) = (
        function_trait_name_from_fn_id(db, &lhs_comparison.function),
        function_trait_name_from_fn_id(db, &rhs_comparison.function),
    );

    let (lhs_var, lhs_litteral) = match (&lhs_comparison.args[0], &lhs_comparison.args[1]) {
        (ExprFunctionCallArg::Value(l_expr_id), ExprFunctionCallArg::Value(r_expr_id)) => {
            match (&arenas.exprs[*l_expr_id], &arenas.exprs[*r_expr_id]) {
                (Expr::Var(var), Expr::Literal(litteral)) => (var, litteral),
                (Expr::Literal(litteral), Expr::Var(var)) => (var, litteral),
                _ => {
                    return;
                }
            }
        }
        _ => {
            return;
        }
    };
    let (rhs_var, rhs_litteral) = match (&rhs_comparison.args[0], &rhs_comparison.args[1]) {
        (ExprFunctionCallArg::Value(l_expr_id), ExprFunctionCallArg::Value(r_expr_id)) => {
            match (&arenas.exprs[*l_expr_id], &arenas.exprs[*r_expr_id]) {
                (Expr::Var(var), Expr::Literal(litteral)) => (var, litteral),
                (Expr::Literal(litteral), Expr::Var(var)) => (var, litteral),
                _ => {
                    return;
                }
            }
        }
        _ => return,
    };

    if !(lhs_var.stable_ptr.lookup(db.upcast()).as_syntax_node().get_text_without_trivia(db.upcast())
        == rhs_var.stable_ptr.lookup(db.upcast()).as_syntax_node().get_text_without_trivia(db.upcast()))
    {
        return;
    }

    if is_impossible_double_comparison(
        &lhs_fn_trait_name,
        &rhs_fn_trait_name,
        lhs_litteral.value.clone(),
        rhs_litteral.value.clone(),
        &expr_logical.op,
    ) {
        diagnostics.push(PluginDiagnostic {
            message: IMPOSSIBLE_COMPARISON.to_string(),
            stable_ptr: expr_logical.stable_ptr.untyped(),
            severity: Severity::Error,
        });
    }
}

fn is_impossible_double_comparison(
    lhs_op: &str,
    rhs_op: &str,
    lhs_int: BigInt,
    rhs_int: BigInt,
    middle_op: &LogicalOperator,
) -> bool {
    match (lhs_op, middle_op, rhs_op) {
        (GT, LogicalOperator::AndAnd, LT) => lhs_int >= rhs_int,
        (GT, LogicalOperator::AndAnd, LE) => lhs_int >= rhs_int,
        (GE, LogicalOperator::AndAnd, LT) => lhs_int >= rhs_int,
        (GE, LogicalOperator::AndAnd, LE) => lhs_int > rhs_int,
        (LT, LogicalOperator::AndAnd, GT) => lhs_int <= rhs_int,
        (LT, LogicalOperator::AndAnd, GE) => lhs_int <= rhs_int,
        (LE, LogicalOperator::AndAnd, GT) => lhs_int <= rhs_int,
        (LE, LogicalOperator::AndAnd, GE) => lhs_int < rhs_int,
        _ => false,
    }
}
