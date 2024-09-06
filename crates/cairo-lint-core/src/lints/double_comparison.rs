use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BinaryOperator, Expr, ExprBinary};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::ids::SyntaxStablePtrId;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const SIMPLIFIABLE_COMPARISON: &str = "This double comparison can be simplified.";
pub const REDUNDANT_COMPARISON: &str =
    "Redundant double comparison found. Consider simplifying to a single comparison.";
pub const CONTRADICTORY_COMPARISON: &str = "This double comparison is contradictory and always false.";

pub fn check_double_comparison(
    db: &dyn SyntaxGroup,
    binary_expr: &ExprBinary,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let lhs_var = extract_variable_from_expr(&binary_expr.lhs(db), db);
    let rhs_var = extract_variable_from_expr(&binary_expr.rhs(db), db);
    let lhs_op = extract_binary_operator_expr(&binary_expr.lhs(db), db);
    let rhs_op = extract_binary_operator_expr(&binary_expr.rhs(db), db);
    let middle_op = binary_expr.op(db);

    if let (Some(lhs_var), Some(rhs_var), Some(lhs_op), Some(rhs_op)) = (lhs_var, rhs_var, lhs_op, rhs_op) {
        if lhs_var == rhs_var {
            if is_simplifiable_double_comparison(&lhs_op, &rhs_op, &middle_op) {
                diagnostics.push(create_diagnostic(
                    SIMPLIFIABLE_COMPARISON,
                    binary_expr.stable_ptr().untyped(),
                    Severity::Warning,
                ));
            } else if is_redundant_double_comparison(&lhs_op, &rhs_op, &middle_op) {
                diagnostics.push(create_diagnostic(
                    REDUNDANT_COMPARISON,
                    binary_expr.stable_ptr().untyped(),
                    Severity::Warning,
                ));
            } else if is_contradictory_double_comparison(&lhs_op, &rhs_op, &middle_op) {
                diagnostics.push(create_diagnostic(
                    CONTRADICTORY_COMPARISON,
                    binary_expr.stable_ptr().untyped(),
                    Severity::Error,
                ));
            }
        }
    }
}

pub fn extract_identifier_from_expr(expr: &Expr, db: &dyn SyntaxGroup) -> Option<String> {
    Some(expr.as_syntax_node().get_text_without_trivia(db))
}

pub fn extract_variable_from_expr(expr: &Expr, db: &dyn SyntaxGroup) -> Option<String> {
    if let Expr::Binary(binary_expr) = expr {
        let lhs = binary_expr.lhs(db);
        let rhs = binary_expr.rhs(db);

        let lhs_text = lhs.as_syntax_node().get_text_without_trivia(db);
        let rhs_text = rhs.as_syntax_node().get_text_without_trivia(db);

        return Some(format!("{} {}", lhs_text, rhs_text));
    }
    None
}

fn create_diagnostic(message: &str, stable_ptr: SyntaxStablePtrId, severity: Severity) -> PluginDiagnostic {
    PluginDiagnostic { stable_ptr, message: message.to_string(), severity }
}

fn is_simplifiable_double_comparison(
    lhs_op: &BinaryOperator,
    rhs_op: &BinaryOperator,
    middle_op: &BinaryOperator,
) -> bool {
    matches!(
        (lhs_op, middle_op, rhs_op),
        (BinaryOperator::LE(_), BinaryOperator::AndAnd(_), BinaryOperator::GE(_))
            | (BinaryOperator::GE(_), BinaryOperator::AndAnd(_), BinaryOperator::LE(_))
            | (BinaryOperator::LT(_), BinaryOperator::OrOr(_), BinaryOperator::EqEq(_))
            | (BinaryOperator::EqEq(_), BinaryOperator::OrOr(_), BinaryOperator::LT(_))
            | (BinaryOperator::GT(_), BinaryOperator::OrOr(_), BinaryOperator::EqEq(_))
            | (BinaryOperator::EqEq(_), BinaryOperator::OrOr(_), BinaryOperator::GT(_))
            | (BinaryOperator::LT(_), BinaryOperator::OrOr(_), BinaryOperator::GT(_))
            | (BinaryOperator::GT(_), BinaryOperator::OrOr(_), BinaryOperator::LT(_))
    )
}

fn is_redundant_double_comparison(
    lhs_op: &BinaryOperator,
    rhs_op: &BinaryOperator,
    middle_op: &BinaryOperator,
) -> bool {
    matches!(
        (lhs_op, middle_op, rhs_op),
        (BinaryOperator::LE(_), BinaryOperator::OrOr(_), BinaryOperator::GE(_))
            | (BinaryOperator::GE(_), BinaryOperator::OrOr(_), BinaryOperator::LE(_))
    )
}

fn is_contradictory_double_comparison(
    lhs_op: &BinaryOperator,
    rhs_op: &BinaryOperator,
    middle_op: &BinaryOperator,
) -> bool {
    matches!(
        (lhs_op, middle_op, rhs_op),
        (BinaryOperator::EqEq(_), BinaryOperator::AndAnd(_), BinaryOperator::LT(_))
            | (BinaryOperator::LT(_), BinaryOperator::AndAnd(_), BinaryOperator::EqEq(_))
            | (BinaryOperator::EqEq(_), BinaryOperator::AndAnd(_), BinaryOperator::GT(_))
            | (BinaryOperator::GT(_), BinaryOperator::AndAnd(_), BinaryOperator::EqEq(_))
            | (BinaryOperator::LT(_), BinaryOperator::AndAnd(_), BinaryOperator::GT(_))
            | (BinaryOperator::GT(_), BinaryOperator::AndAnd(_), BinaryOperator::LT(_))
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

pub fn extract_binary_operator_expr(expr: &Expr, db: &dyn SyntaxGroup) -> Option<BinaryOperator> {
    if let Expr::Binary(binary_op) = expr { Some(binary_op.op(db)) } else { None }
}
