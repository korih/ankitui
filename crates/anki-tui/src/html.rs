/// Converts Anki's rendered card HTML (question/answer fields, cloze already
/// resolved) into plain text sized for the given terminal width.
pub fn html_to_text(html: &str, width: u16) -> String {
    let width = width.max(20) as usize;
    html2text::from_read(html.as_bytes(), width).unwrap_or_else(|_| html.to_string())
}
