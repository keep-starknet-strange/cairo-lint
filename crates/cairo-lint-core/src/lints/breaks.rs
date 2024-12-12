use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, StatementBreak};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use if_chain::if_chain;

pub const BREAK_UNIT: &str =
    "unnecessary double parentheses found after break. Consider removing them.";

pub const ALLOWED: [&str; 1] = [LINT_NAME];
const LINT_NAME: &str = "break_unit";

pub fn check_break(
    db: &dyn SemanticGroup,
    stmt_break: &StatementBreak,
    arenas: &Arenas,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let mut current_node = stmt_break.stable_ptr.lookup(db.upcast()).as_syntax_node();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }
    if_chain! {
        if let Some(expr) = stmt_break.expr_option;
        if arenas.exprs[expr].ty().is_unit(db);
        then {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: stmt_break.stable_ptr.untyped(),
                message: BREAK_UNIT.to_string(),
                severity: Severity::Warning,
            });
        }
    }
}
