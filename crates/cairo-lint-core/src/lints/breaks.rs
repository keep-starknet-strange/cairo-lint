use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::SyntaxNode;

pub const BREAK_UNIT: &str = "unnecessary double parentheses found after break. Consider removing them.";

pub fn check_break(db: &dyn SyntaxGroup, node: SyntaxNode, diagnostics: &mut Vec<PluginDiagnostic>) {
    if let Some(node) = node.parent()
        && node.has_attr_with_arg(db, "allow", "break_unit")
    {
        return;
    }
    if node.clone().get_text_without_trivia(db).ends_with("();") {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: node.stable_ptr(),
            message: BREAK_UNIT.to_string(),
            severity: Severity::Warning,
        });
    }
}
