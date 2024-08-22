use annotate_snippets::{Level, Renderer, Snippet};
use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_diagnostics::{DiagnosticEntry, Severity};
use cairo_lang_filesystem::db::FilesGroup;
use cairo_lang_semantic::SemanticDiagnostic;
use cairo_lang_utils::Upcast;

pub fn format_diagnostic<'a>(diagnostic: &'a SemanticDiagnostic, db: &'a RootDatabase, renderer: &Renderer) -> String {
    let location = diagnostic.location(db.upcast());
    let file_id = location.file_id;
    let span = location.span;
    let file_location = span.position_in_file(db.upcast(), file_id).unwrap();
    let level = match diagnostic.severity() {
        Severity::Warning => Level::Warning,
        Severity::Error => Level::Error,
    };
    let res = renderer
        .render(
            level.title(&diagnostic.format(db)).snippet(
                Snippet::source(db.file_content(file_id).unwrap().as_ref())
                    .line_start(file_location.start.line)
                    .origin(&file_id.full_path(db.upcast()))
                    .fold(true)
                    .annotation(level.span(span.to_str_range())),
            ),
        )
        .to_string();
    format!("{}\n", res)
}
