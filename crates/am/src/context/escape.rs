/// Escape characters that would break a markdown table cell.
///
/// Currently only `|` is escaped (most common in shell command bodies). Other
/// markdown specials (backticks, asterisks, etc.) are left verbatim — they
/// render fine inside table cells and the spec calls for verbatim output.
pub(super) fn escape_md_cell(s: &str) -> String {
    s.replace('|', r"\|")
}
