use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BinaryOperator, Expr, ExprBinary};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const DOUBLE_COMPARISON: &str = "redundant double comparison found. Consider simplifying to a single comparison.";

pub fn check_double_comparison(
    db: &dyn SyntaxGroup,
    binary_expr: &ExprBinary,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let lhs = binary_expr.lhs(db);
    let rhs = binary_expr.rhs(db);
    let middle_op = binary_expr.op(db);

    if let (Some(lhs_op), Some(rhs_op)) =
        (extract_binary_operator_expr(&lhs, db), extract_binary_operator_expr(&rhs, db))
    {
        let lhs_var = extract_variable_from_expr(lhs.as_syntax_node().get_text(db));
        let rhs_var = extract_variable_from_expr(rhs.as_syntax_node().get_text(db));

        if lhs_var == rhs_var {
            if is_redundant_double_comparison(&lhs_op, &rhs_op, &middle_op) {
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: binary_expr.stable_ptr().untyped(),
                    message: DOUBLE_COMPARISON.to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }
}

pub fn extract_variable_from_expr(text: String) -> String {
    let parts: Vec<&str> = text.split_whitespace().collect();
    match parts.as_slice() {
        [var1, _, var2] => format!("{} {}", var1, var2),
        _ => text,
    }
}

fn is_redundant_double_comparison(
    lhs_op: &BinaryOperator,
    rhs_op: &BinaryOperator,
    middle_op: &BinaryOperator,
) -> bool {
    matches!(
        (lhs_op, rhs_op, middle_op),
        (BinaryOperator::EqEq(_), BinaryOperator::LT(_), BinaryOperator::AndAnd(_))
            | (BinaryOperator::LT(_), BinaryOperator::EqEq(_), BinaryOperator::AndAnd(_))
            | (BinaryOperator::EqEq(_), BinaryOperator::GT(_), BinaryOperator::AndAnd(_))
            | (BinaryOperator::GT(_), BinaryOperator::EqEq(_), BinaryOperator::AndAnd(_))
            | (BinaryOperator::LE(_), BinaryOperator::GE(_), BinaryOperator::AndAnd(_))
            | (BinaryOperator::GE(_), BinaryOperator::LE(_), BinaryOperator::AndAnd(_))
            | (BinaryOperator::LT(_), BinaryOperator::GT(_), BinaryOperator::AndAnd(_))
            | (BinaryOperator::GT(_), BinaryOperator::LT(_), BinaryOperator::AndAnd(_))
            | (BinaryOperator::EqEq(_), BinaryOperator::LT(_), BinaryOperator::OrOr(_))
            | (BinaryOperator::LT(_), BinaryOperator::EqEq(_), BinaryOperator::OrOr(_))
            | (BinaryOperator::EqEq(_), BinaryOperator::GT(_), BinaryOperator::OrOr(_))
            | (BinaryOperator::GT(_), BinaryOperator::EqEq(_), BinaryOperator::OrOr(_))
            | (BinaryOperator::LE(_), BinaryOperator::GE(_), BinaryOperator::OrOr(_))
            | (BinaryOperator::GE(_), BinaryOperator::LE(_), BinaryOperator::OrOr(_))
            | (BinaryOperator::LT(_), BinaryOperator::GT(_), BinaryOperator::OrOr(_))
            | (BinaryOperator::GT(_), BinaryOperator::LT(_), BinaryOperator::OrOr(_))
    )
}

pub fn operator_to_replace(lhs_op: BinaryOperator) -> &'static str {
    match lhs_op {
        BinaryOperator::EqEq(_) => "==",
        BinaryOperator::GT(_) => ">",
        BinaryOperator::LT(_) => "<",
        BinaryOperator::GE(_) => ">=",
        BinaryOperator::LE(_) => "<=",
        _ => "",
    }
}

pub fn determine_simplified_operator(
    lhs_op: &BinaryOperator,
    rhs_op: &BinaryOperator,
    middle_op: &BinaryOperator,
) -> Option<&'static str> {
    match (lhs_op, rhs_op, middle_op) {
        (BinaryOperator::EqEq(_), BinaryOperator::LT(_), BinaryOperator::AndAnd(_))
        | (BinaryOperator::LT(_), BinaryOperator::EqEq(_), BinaryOperator::AndAnd(_)) => Some("<="),

        (BinaryOperator::EqEq(_), BinaryOperator::GT(_), BinaryOperator::AndAnd(_))
        | (BinaryOperator::GT(_), BinaryOperator::EqEq(_), BinaryOperator::AndAnd(_)) => Some(">="),

        (BinaryOperator::LT(_), BinaryOperator::GT(_), BinaryOperator::AndAnd(_))
        | (BinaryOperator::GT(_), BinaryOperator::LT(_), BinaryOperator::AndAnd(_)) => Some("!="),

        (BinaryOperator::LE(_), BinaryOperator::GE(_), BinaryOperator::AndAnd(_))
        | (BinaryOperator::GE(_), BinaryOperator::LE(_), BinaryOperator::AndAnd(_)) => Some("=="),

        (BinaryOperator::EqEq(_), BinaryOperator::LT(_), BinaryOperator::OrOr(_))
        | (BinaryOperator::LT(_), BinaryOperator::EqEq(_), BinaryOperator::OrOr(_)) => Some("<="),

        (BinaryOperator::EqEq(_), BinaryOperator::GT(_), BinaryOperator::OrOr(_))
        | (BinaryOperator::GT(_), BinaryOperator::EqEq(_), BinaryOperator::OrOr(_)) => Some(">="),

        (BinaryOperator::LT(_), BinaryOperator::GT(_), BinaryOperator::OrOr(_))
        | (BinaryOperator::GT(_), BinaryOperator::LT(_), BinaryOperator::OrOr(_)) => Some("!="),

        (BinaryOperator::LE(_), BinaryOperator::GE(_), BinaryOperator::OrOr(_))
        | (BinaryOperator::GE(_), BinaryOperator::LE(_), BinaryOperator::OrOr(_)) => Some("=="),

        _ => None,
    }
}

pub fn extract_binary_operator_expr(expr: &Expr, db: &dyn SyntaxGroup) -> Option<BinaryOperator> {
    if let Expr::Binary(binary_op) = expr { Some(binary_op.op(db)) } else { None }
}
