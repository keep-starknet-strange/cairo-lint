use std::collections::HashSet;

use cairo_lang_defs::ids::{FunctionWithBodyId, TopLevelLanguageElementId};
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Expr, ExprFunctionCallArg, ExprLogicalOperator, FunctionId, LogicalOperator};
use cairo_lang_syntax::node::ast::{BinaryOperator, Expr as AstExpr};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

use crate::lints::{EQ, GE, GT, LE, LT};

pub const SIMPLIFIABLE_COMPARISON: &str = "This double comparison can be simplified.";
pub const REDUNDANT_COMPARISON: &str =
    "Redundant double comparison found. Consider simplifying to a single comparison.";
pub const CONTRADICTORY_COMPARISON: &str = "This double comparison is contradictory and always false.";

pub const ALLOWED: [&str; 3] =
    [redundant_comaprison::LINT_NAME, contradictory_comparison::LINT_NAME, simplifiable_comparison::LINT_NAME];

mod redundant_comaprison {
    pub(super) const LINT_NAME: &str = "redundant_comparison";
}
mod contradictory_comparison {
    pub(super) const LINT_NAME: &str = "contradictory_comparison";
}
mod simplifiable_comparison {
    pub(super) const LINT_NAME: &str = "simplifiable_comparison";
}

pub fn check_double_comparison(
    db: &dyn SemanticGroup,
    expr_logical: &ExprLogicalOperator,
    arenas: &Arenas,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let (mut ignore_redundant, mut ignore_contradictory, mut ignore_simplifiable) = (false, false, false);
    // Checks if the lint is allowed in an upper scope.
    let mut current_node = expr_logical.stable_ptr.lookup(db.upcast()).as_syntax_node();
    let syntax_db = db.upcast();
    while let Some(node) = current_node.parent() {
        ignore_redundant |= node.has_attr_with_arg(syntax_db, "allow", redundant_comaprison::LINT_NAME);
        ignore_contradictory |= node.has_attr_with_arg(syntax_db, "allow", contradictory_comparison::LINT_NAME);
        ignore_simplifiable |= node.has_attr_with_arg(syntax_db, "allow", simplifiable_comparison::LINT_NAME);
        current_node = node;
    }

    let Expr::FunctionCall(lhs_comparison) = &arenas.exprs[expr_logical.lhs] else {
        return;
    };
    // If it's not 2 args it cannot be a regular comparison
    if lhs_comparison.args.len() != 2 {
        return;
    }

    let Expr::FunctionCall(rhs_comparison) = &arenas.exprs[expr_logical.rhs] else {
        return;
    };
    // If it's not 2 args it cannot be a regular comparison
    if rhs_comparison.args.len() != 2 {
        return;
    }
    // Get the full name of the function used (trait name)
    let (lhs_fn_trait_name, rhs_fn_trait_name) = (
        function_trait_name_from_fn_id(db, &lhs_comparison.function),
        function_trait_name_from_fn_id(db, &rhs_comparison.function),
    );

    // The comparison functions don't work with refs so should only be value
    let (llhs, rlhs) = match (&lhs_comparison.args[0], &lhs_comparison.args[1]) {
        (ExprFunctionCallArg::Value(l_expr_id), ExprFunctionCallArg::Value(r_expr_id)) => {
            (&arenas.exprs[*l_expr_id], &arenas.exprs[*r_expr_id])
        }
        _ => {
            return;
        }
    };
    let (lrhs, rrhs) = match (&rhs_comparison.args[0], &rhs_comparison.args[1]) {
        (ExprFunctionCallArg::Value(l_expr_id), ExprFunctionCallArg::Value(r_expr_id)) => {
            (&arenas.exprs[*l_expr_id], &arenas.exprs[*r_expr_id])
        }
        _ => return,
    };
    // Get all the operands
    let llhs_var = llhs.stable_ptr().lookup(db.upcast()).as_syntax_node().get_text_without_trivia(db.upcast());
    let rlhs_var = rlhs.stable_ptr().lookup(db.upcast()).as_syntax_node().get_text_without_trivia(db.upcast());
    let lrhs_var = lrhs.stable_ptr().lookup(db.upcast()).as_syntax_node().get_text_without_trivia(db.upcast());
    let rrhs_var = rrhs.stable_ptr().lookup(db.upcast()).as_syntax_node().get_text_without_trivia(db.upcast());
    // Put them in a hashset to check equality without order
    let lhs: HashSet<String> = HashSet::from_iter([llhs_var, rlhs_var]);
    let rhs: HashSet<String> = HashSet::from_iter([lrhs_var, rrhs_var]);
    if lhs != rhs {
        return;
    }

    // TODO: support other expressions like tuples and literals
    let should_return = match (llhs, rlhs) {
        (Expr::Snapshot(llhs), Expr::Snapshot(rlhs)) => {
            matches!(arenas.exprs[llhs.inner], Expr::FunctionCall(_))
                || matches!(arenas.exprs[rlhs.inner], Expr::FunctionCall(_))
        }
        (Expr::Var(_), Expr::Var(_)) => false,
        (Expr::Snapshot(llhs), Expr::Var(_)) => {
            matches!(arenas.exprs[llhs.inner], Expr::FunctionCall(_))
        }
        (Expr::Var(_), Expr::Snapshot(rlhs)) => {
            matches!(arenas.exprs[rlhs.inner], Expr::FunctionCall(_))
        }
        _ => return,
    };
    if should_return {
        return;
    }

    if !ignore_simplifiable
        && is_simplifiable_double_comparison(&lhs_fn_trait_name, &rhs_fn_trait_name, &expr_logical.op)
    {
        diagnostics.push(PluginDiagnostic {
            message: SIMPLIFIABLE_COMPARISON.to_string(),
            stable_ptr: expr_logical.stable_ptr.untyped(),
            severity: Severity::Warning,
        });
    } else if !ignore_redundant
        && is_redundant_double_comparison(&lhs_fn_trait_name, &rhs_fn_trait_name, &expr_logical.op)
    {
        diagnostics.push(PluginDiagnostic {
            message: REDUNDANT_COMPARISON.to_string(),
            stable_ptr: expr_logical.stable_ptr.untyped(),
            severity: Severity::Warning,
        });
    } else if !ignore_contradictory
        && is_contradictory_double_comparison(&lhs_fn_trait_name, &rhs_fn_trait_name, &expr_logical.op)
    {
        diagnostics.push(PluginDiagnostic {
            message: CONTRADICTORY_COMPARISON.to_string(),
            stable_ptr: expr_logical.stable_ptr.untyped(),
            severity: Severity::Error,
        });
    }
}
pub(crate) fn function_trait_name_from_fn_id(db: &dyn SemanticGroup, function: &FunctionId) -> String {
    let Ok(Some(func_id)) = function.get_concrete(db).body(db) else {
        return String::new();
    };
    // Get the trait function id of the function (if there's none it means it cannot be a call to
    // a corelib trait)
    let trait_fn_id = match func_id.function_with_body_id(db) {
        FunctionWithBodyId::Impl(func) => db.impl_function_trait_function(func).unwrap(),
        FunctionWithBodyId::Trait(func) => func,
        _ => return String::new(),
    };
    // From the trait function id get the trait name and check if it's the corelib `BitAnd`
    trait_fn_id.full_path(db.upcast())
}

fn is_simplifiable_double_comparison(lhs_op: &str, rhs_op: &str, middle_op: &LogicalOperator) -> bool {
    matches!(
        (lhs_op, middle_op, rhs_op),
        (LE, LogicalOperator::AndAnd, GE)
            | (GE, LogicalOperator::AndAnd, LE)
            | (LT, LogicalOperator::OrOr, EQ)
            | (EQ, LogicalOperator::OrOr, LT)
            | (GT, LogicalOperator::OrOr, EQ)
            | (EQ, LogicalOperator::OrOr, GT)
    )
}

fn is_redundant_double_comparison(lhs_op: &str, rhs_op: &str, middle_op: &LogicalOperator) -> bool {
    matches!(
        (lhs_op, middle_op, rhs_op),
        (LE, LogicalOperator::OrOr, GE)
            | (GE, LogicalOperator::OrOr, LE)
            | (LT, LogicalOperator::OrOr, GT)
            | (GT, LogicalOperator::OrOr, LT)
    )
}

fn is_contradictory_double_comparison(lhs_op: &str, rhs_op: &str, middle_op: &LogicalOperator) -> bool {
    matches!(
        (lhs_op, middle_op, rhs_op),
        (EQ, LogicalOperator::AndAnd, LT)
            | (LT, LogicalOperator::AndAnd, EQ)
            | (EQ, LogicalOperator::AndAnd, GT)
            | (GT, LogicalOperator::AndAnd, EQ)
            | (LT, LogicalOperator::AndAnd, GT)
            | (GT, LogicalOperator::AndAnd, LT)
            | (GT, LogicalOperator::AndAnd, GE)
            | (LE, LogicalOperator::AndAnd, GT)
    )
}

pub fn operator_to_replace(lhs_op: BinaryOperator) -> Option<&'static str> {
    match lhs_op {
        BinaryOperator::EqEq(_) => Some("=="),
        BinaryOperator::GT(_) => Some(">"),
        BinaryOperator::LT(_) => Some("<"),
        BinaryOperator::GE(_) => Some(">="),
        BinaryOperator::LE(_) => Some("<="),
        _ => None,
    }
}

pub fn determine_simplified_operator(
    lhs_op: &BinaryOperator,
    rhs_op: &BinaryOperator,
    middle_op: &BinaryOperator,
) -> Option<&'static str> {
    match (lhs_op, middle_op, rhs_op) {
        (BinaryOperator::LE(_), BinaryOperator::AndAnd(_), BinaryOperator::GE(_))
        | (BinaryOperator::GE(_), BinaryOperator::AndAnd(_), BinaryOperator::LE(_)) => Some("=="),

        (BinaryOperator::LT(_), BinaryOperator::OrOr(_), BinaryOperator::EqEq(_))
        | (BinaryOperator::EqEq(_), BinaryOperator::OrOr(_), BinaryOperator::LT(_)) => Some("<="),

        (BinaryOperator::GT(_), BinaryOperator::OrOr(_), BinaryOperator::EqEq(_))
        | (BinaryOperator::EqEq(_), BinaryOperator::OrOr(_), BinaryOperator::GT(_)) => Some(">="),

        (BinaryOperator::LT(_), BinaryOperator::OrOr(_), BinaryOperator::GT(_))
        | (BinaryOperator::GT(_), BinaryOperator::OrOr(_), BinaryOperator::LT(_)) => Some("!="),

        _ => None,
    }
}

pub fn extract_binary_operator_expr(expr: &AstExpr, db: &dyn SyntaxGroup) -> Option<BinaryOperator> {
    if let AstExpr::Binary(binary_op) = expr {
        Some(binary_op.op(db))
    } else {
        None
    }
}
