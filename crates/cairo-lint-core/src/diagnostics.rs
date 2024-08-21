use annotate_snippets::{Level, Renderer, Snippet};

pub fn format_diagnostic(
    file_name: &str,
    source_code: &str,
    start_line: usize,
    start_col: usize,
    end_col: usize,
    message: &str,
) -> String {
    let snippet = Snippet::source(source_code)
        .line_start(start_line)
        .origin(file_name)
        .fold(true)
        .annotation(
            Level::Warning
                .span(start_col..end_col)
                .label(message)
        );

    let message = Level::Warning
        .title(message)
        .snippet(snippet);

    let renderer = Renderer::styled();
    let rendered_message = renderer.render(message).to_string();
    rendered_message
}
