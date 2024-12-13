use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Condition, Expr, ExprWhile};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

const INEFFICIENT_WHILE_COMP_MESSAGE: &str = "using [`<`, `<=`, `>=`, `>`] exit conditions is inefficient. Consider \
                                              switching to `!=` or using ArrayTrait::multi_pop_front.";

pub const ALLOWED: [&str; 1] = [LINT_NAME];
const LINT_NAME: &str = "inefficient_while_comp";
// Match all types implementing PartialOrd
const PARTIAL_ORD_PATTERNS: [&str; 4] = [
    "PartialOrd::lt\"",
    "PartialOrd::le\"",
    "PartialOrd::gt\"",
    "PartialOrd::ge\"",
];

pub fn check_inefficient_while_comp(
    db: &dyn SemanticGroup,
    expr_while: &ExprWhile,
    diagnostics: &mut Vec<PluginDiagnostic>,
    arenas: &Arenas,
) {
    // Checks if the lint is allowed in an upper scope.
    let mut current_node = expr_while.stable_ptr.lookup(db.upcast()).as_syntax_node();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }
    // It might be a false positive, because there can be cases when:
    //  - The rhs arguments is changed in the loop body
    //  - The lhs argument can "skip" the moment where lhs == rhs
    if let Condition::BoolExpr(expr_cond) = expr_while.condition {
        check_expression(db, &arenas.exprs[expr_cond], diagnostics, arenas);
    }
}

fn check_expression(
    db: &dyn SemanticGroup,
    expr: &Expr,
    diagnostics: &mut Vec<PluginDiagnostic>,
    arenas: &Arenas,
) {
    match expr {
        Expr::FunctionCall(func_call) => {
            let func_name = func_call.function.name(db);
            if PARTIAL_ORD_PATTERNS.iter().any(|p| func_name.ends_with(p)) {
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: func_call.stable_ptr.into(),
                    message: INEFFICIENT_WHILE_COMP_MESSAGE.to_owned(),
                    severity: Severity::Warning,
                });
            }
        }
        Expr::LogicalOperator(expr_logical) => {
            check_expression(db, &arenas.exprs[expr_logical.lhs], diagnostics, arenas);
            check_expression(db, &arenas.exprs[expr_logical.rhs], diagnostics, arenas);
        }
        _ => {}
    }
}
