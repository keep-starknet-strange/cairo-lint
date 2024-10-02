use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{BinaryOperator, ExprBinary};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

pub const BITWISE_FOR_PARITY: &str =
    "You seem to be trying to use `&` for parity check. Consider using `DivRem::div_rem()` instead.";

pub const ALLOWED: [&str; 1] = [LINT_NAME];
const LINT_NAME: &str = "bitwise_for_parity_check";

pub fn check_bitwise_for_parity(db: &dyn SyntaxGroup, node: &ExprBinary, diagnostics: &mut Vec<PluginDiagnostic>) {
    let rhs = node.rhs(db).as_syntax_node().get_text_without_trivia(db);
    let op = node.op(db);

    if matches!(op, BinaryOperator::And(_)) && rhs == *"1".to_string() {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: node.as_syntax_node().stable_ptr(),
            message: BITWISE_FOR_PARITY.to_string(),
            severity: Severity::Warning,
        });
    }
}
