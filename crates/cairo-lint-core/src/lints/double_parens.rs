use std::path::PathBuf;

use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::Expr;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use cairo_lang_filesystem::ids::FileId;
use crate::diagnostics::format_diagnostic;

pub const DOUBLE_PARENS: &str = "unnecessary double parentheses found. Consider removing them.";

pub fn check_double_parens(db: &dyn SyntaxGroup, expr: &Expr, diagnostics: &mut Vec<PluginDiagnostic>) {
    let is_double_parens = if let Expr::Parenthesized(parenthesized_expr) = expr {
        matches!(parenthesized_expr.expr(db), Expr::Parenthesized(_) | Expr::Tuple(_))
    } else {
        false
    };
    let source_code ="todo";
    
    // db should be FileGroup but breaks isDoubleParens if changed
    // let file_id = FileId::new(db, PathBuf::from("path/to/file"));

    let file_id= 0;

    if is_double_parens {
        let diagnostic = PluginDiagnostic {
            stable_ptr: expr.stable_ptr().untyped(),
            message: DOUBLE_PARENS.to_string(),
            severity: Severity::Warning,
        };
        diagnostics.push(diagnostic);

        // Format and print the diagnostic
        let formatted_message = format_diagnostic(diagnostic, db, source_code, file_id);
        println!("{}", formatted_message);
    }
}
