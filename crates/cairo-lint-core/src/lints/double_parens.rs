use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::Expr;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use cairo_lang_filesystem::span::TextSpan;
use cairo_lang_filesystem::ids::FileId;
use crate::diagnostics::format_diagnostic;

pub const DOUBLE_PARENS: &str = "unnecessary double parentheses found. Consider removing them.";

pub fn check_double_parens(
    db: &dyn SyntaxGroup,
    expr: &Expr,
    diagnostics: &mut Vec<PluginDiagnostic>,
    span: TextSpan,
    file_id: FileId
) {
    let is_double_parens = if let Expr::Parenthesized(parenthesized_expr) = expr {
        matches!(parenthesized_expr.expr(db), Expr::Parenthesized(_) | Expr::Tuple(_))
    } else {
        false
    };

    if is_double_parens {
        if let Some(span) = span.position_in_file(db.upcast(), file_id) {
            let line_start = span.start.line;
            let col_start = span.start.col;
            let col_end = span.end.col;

            let file_content = match db.file_content(file_id) {
                Some(content) => content.to_owned(),
                None => return,
            };

            let snippet = &file_content[span.start.line..span.end.line];

            let formatted_message = format_diagnostic(
                "todo: get file name",
                snippet,
                line_start,
                col_start,
                col_end,
                DOUBLE_PARENS,
            );
            diagnostics.push(PluginDiagnostic {
                stable_ptr: expr.stable_ptr().untyped(),
                message: formatted_message,
                severity: Severity::Warning,
            });
        }
    }
}
