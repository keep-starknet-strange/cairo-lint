use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BinaryOperator, Expr, ExprBinary};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::kind::SyntaxKind;
use cairo_lang_syntax::node::TypedSyntaxNode;

pub const BOOL_COMPARISON: &str = "Unnecessary comparison with a boolean value. Use the variable directly.";

pub fn generate_fixed_text_for_comparison(db: &dyn SyntaxGroup, lhs: &str, rhs: &str, node: ExprBinary) -> String {
    let op_kind = node.op(db).as_syntax_node().kind(db);
    let lhs = lhs.trim();
    let rhs = rhs.trim();

    let result = match (lhs, rhs, op_kind) {
        // lhs
        ("false", _, SyntaxKind::TerminalEqEq | SyntaxKind::TokenEqEq) => format!("!{} ", rhs),
        ("true", _, SyntaxKind::TerminalEqEq | SyntaxKind::TokenEqEq) => format!("{} ", rhs),
        ("false", _, SyntaxKind::TerminalNeq) => format!("!{} ", rhs),
        ("true", _, SyntaxKind::TerminalNeq) => format!("!{} ", rhs),

        // rhs
        (_, "false", SyntaxKind::TerminalEqEq | SyntaxKind::TokenEqEq) => format!("!{} ", lhs),
        (_, "true", SyntaxKind::TerminalEqEq | SyntaxKind::TokenEqEq) => format!("{} ", lhs),
        (_, "false", SyntaxKind::TerminalNeq) => format!("!{} ", lhs),
        (_, "true", SyntaxKind::TerminalNeq) => format!("!{} ", lhs),

        _ => node.as_syntax_node().get_text(db).to_string(),
    };

    result
}

pub fn check_bool_comparison(db: &dyn SyntaxGroup, node: ExprBinary, diagnostics: &mut Vec<PluginDiagnostic>) {
    let lhs = node.lhs(db);
    let op = node.op(db);
    let rhs = node.rhs(db);

    let is_comparison_operator = matches!(op, BinaryOperator::EqEq(_) | BinaryOperator::Neq(_));

    fn is_bool_literal(expr: &Expr) -> bool {
        matches!(expr, Expr::True(_) | Expr::False(_))
    }

    if is_comparison_operator && (is_bool_literal(&lhs) || is_bool_literal(&rhs)) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: node.as_syntax_node().stable_ptr(),
            message: BOOL_COMPARISON.to_string(),
            severity: Severity::Warning,
        });
    }
}
