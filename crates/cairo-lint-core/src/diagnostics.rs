use annotate_snippets::{Level, Renderer, Snippet};
use cairo_lang_syntax::node::{db::SyntaxGroup, SyntaxNode};
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_filesystem::ids::FileId;

// Get the start and end positions from the syntax node
fn get_node_position(node: &SyntaxNode, db: &dyn SyntaxGroup, file_id: FileId) -> (usize, usize, usize) {
    let span = node.span(db);

    if let Some(span) = span.position_in_file(db.upcast(), file_id) {
        let start_line = span.start.line;
        let start_col = span.start.col;
        let end_col = span.end.col;

        (start_line, start_col, end_col)
    } else {
        (0, 0, 0) // Default return if span does not have a position in the file
    }
}

pub fn format_diagnostic(
    diagnostic: PluginDiagnostic,
    db: &dyn SyntaxGroup,
    source_code: &str,
    file_id: i32,
) -> String {
    // Look up the node for the diagnostic
    let node = diagnostic.stable_ptr.lookup(db);

    // Get the start and end positions
    let (start_line, start_col, end_col) = get_node_position(&node, db, file_id);

    // Create the snippet
    let snippet = Snippet::source(source_code)
        .line_start(start_line)
        .origin("example_file.cairo")
        .fold(true)
        .annotation(
            Level::Warning
                .span(start_col..end_col)
                .label(&diagnostic.message)
        );

    // Format the message
    let message = Level::Warning
        .title(&diagnostic.message)
        .snippet(snippet);

    // Use the renderer to style the message
    let renderer = Renderer::styled();
    let rendered_message = renderer.render(message).to_string();
    rendered_message
}
