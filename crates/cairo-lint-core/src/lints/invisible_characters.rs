use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::Expr;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const WHITESPACE_DETECTED: &str = "Whitespace detected.";
pub const INVISIBLE_CHARACTER_DETECTED: &str = "Invisible character detected.";

pub fn check_invisible_characters(expr: &Expr, db: &dyn SyntaxGroup, diagnostics: &mut Vec<PluginDiagnostic>) {
    let syntax_node = expr.as_syntax_node();
    let text = syntax_node.get_text(db);
    if text.chars().any(char::is_whitespace) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr.stable_ptr().untyped(),
            message: WHITESPACE_DETECTED.to_string(),
            severity: Severity::Warning,
        });
    }
    let invisible_chars = ['\u{200B}', '\u{00AD}', '\u{200C}', '\u{200D}', '\u{202C}', '\u{FEFF}'];
    if text.chars().any(|c| invisible_chars.contains(&c)) {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr.stable_ptr().untyped(),
            message: INVISIBLE_CHARACTER_DETECTED.to_string(),
            severity: Severity::Error,
        });
    }
}