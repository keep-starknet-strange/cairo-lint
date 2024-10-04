use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{Expr, ExprBinary};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

pub const INT_PLUS_ONE: &str = "Inefficient comparison: Consider using 'x > y' instead of 'x >= y + 1' or similar.";

pub fn check_int_plus_one(db: &dyn SyntaxGroup, expr_binary: &ExprBinary, diagnostics: &mut Vec<PluginDiagnostic>) {
    let operator_text = expr_binary.op(db).as_syntax_node().get_text_without_trivia(db);
    if operator_text != ">=" && operator_text != "<=" {
        return;
    }

    if is_int_plus_one_expr(db, expr_binary) || is_int_minus_one_expr(db, expr_binary) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_binary.as_syntax_node().stable_ptr(),
            message: INT_PLUS_ONE.to_string(),
            severity: Severity::Warning,
        });
    }
}

fn is_int_plus_one_expr(db: &dyn SyntaxGroup, expr_binary: &ExprBinary) -> bool {
    if let Expr::Binary(rhs_expr) = expr_binary.rhs(db) {
        let operator_text = rhs_expr.op(db).as_syntax_node().get_text_without_trivia(db);
        return operator_text == "+" && rhs_expr.rhs(db).as_syntax_node().get_text_without_trivia(db) == "1";
    }
    false
}

fn is_int_minus_one_expr(db: &dyn SyntaxGroup, expr_binary: &ExprBinary) -> bool {
    if let Expr::Binary(lhs_expr) = expr_binary.lhs(db) {
        let operator_text = lhs_expr.op(db).as_syntax_node().get_text_without_trivia(db);
        return operator_text == "-" && lhs_expr.rhs(db).as_syntax_node().get_text_without_trivia(db) == "1";
    }
    false
}
