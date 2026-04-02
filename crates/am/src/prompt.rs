use std::io::{BufRead, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Answer {
    Yes,
    No,
    Cancel,
}

/// Strip ANSI CSI escape sequences (e.g. `\x1b[?1l`) from the input.
fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                // Skip parameter/intermediate bytes until final byte (@ through ~)
                for c in chars.by_ref() {
                    if c.is_ascii() && ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Print a yes/no(/cancel) question to stderr and read the answer from `reader`.
///
/// - `default` determines which option is uppercase in the prompt and is returned
///   when the user presses Enter (empty input).
/// - `show_cancel` adds a `/c` cancel option to the hint.
/// - Unrecognised input is treated as `No` (only explicit "y"/"yes"/Enter-on-default-yes proceeds).
/// - EOF returns `Cancel` (no consent without explicit input).
pub fn ask_user(
    question: &str,
    default: Answer,
    show_cancel: bool,
    reader: &mut dyn BufRead,
) -> std::io::Result<Answer> {
    let hint = match (default, show_cancel) {
        (Answer::Yes, false) => "[Y/n]",
        (Answer::Yes, true) => "[Y/n/c]",
        (Answer::No, false) => "[y/N]",
        (Answer::No, true) => "[y/N/c]",
        (Answer::Cancel, _) => "[y/n/C]",
    };
    eprint!("{question} {hint} ");
    std::io::stderr().flush()?;

    let mut input = String::new();
    let bytes = reader.read_line(&mut input)?;
    if bytes == 0 {
        // EOF — input stream ended, treat as cancel
        return Ok(Answer::Cancel);
    }
    let clean: String = strip_ansi(&input)
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_lowercase();
    match clean.as_str() {
        "" => Ok(default),
        "y" | "yes" => Ok(Answer::Yes),
        "n" | "no" => Ok(Answer::No),
        "c" | "cancel" if show_cancel => Ok(Answer::Cancel),
        _ => Ok(Answer::No),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn yes_default_enter_returns_yes() {
        let mut reader = Cursor::new(b"\n");
        assert_eq!(
            ask_user("Q?", Answer::Yes, false, &mut reader).unwrap(),
            Answer::Yes
        );
    }

    #[test]
    fn yes_default_n_returns_no() {
        let mut reader = Cursor::new(b"n\n");
        assert_eq!(
            ask_user("Q?", Answer::Yes, false, &mut reader).unwrap(),
            Answer::No
        );
    }

    #[test]
    fn yes_default_uppercase_n_returns_no() {
        let mut reader = Cursor::new(b"N\n");
        assert_eq!(
            ask_user("Q?", Answer::Yes, false, &mut reader).unwrap(),
            Answer::No
        );
    }

    #[test]
    fn yes_default_eof_returns_cancel() {
        let mut reader = Cursor::new(b"");
        assert_eq!(
            ask_user("Q?", Answer::Yes, false, &mut reader).unwrap(),
            Answer::Cancel
        );
    }

    #[test]
    fn no_default_enter_returns_no() {
        let mut reader = Cursor::new(b"\n");
        assert_eq!(
            ask_user("Q?", Answer::No, false, &mut reader).unwrap(),
            Answer::No
        );
    }

    #[test]
    fn no_default_y_returns_yes() {
        let mut reader = Cursor::new(b"y\n");
        assert_eq!(
            ask_user("Q?", Answer::No, false, &mut reader).unwrap(),
            Answer::Yes
        );
    }

    #[test]
    fn no_default_eof_returns_cancel() {
        let mut reader = Cursor::new(b"");
        assert_eq!(
            ask_user("Q?", Answer::No, false, &mut reader).unwrap(),
            Answer::Cancel
        );
    }

    #[test]
    fn cancel_option_c_returns_cancel() {
        let mut reader = Cursor::new(b"c\n");
        assert_eq!(
            ask_user("Q?", Answer::No, true, &mut reader).unwrap(),
            Answer::Cancel
        );
    }

    #[test]
    fn cancel_word_returns_cancel() {
        let mut reader = Cursor::new(b"cancel\n");
        assert_eq!(
            ask_user("Q?", Answer::No, true, &mut reader).unwrap(),
            Answer::Cancel
        );
    }

    #[test]
    fn cancel_not_shown_c_returns_no() {
        let mut reader = Cursor::new(b"c\n");
        assert_eq!(
            ask_user("Q?", Answer::Yes, false, &mut reader).unwrap(),
            Answer::No,
            "unrecognised input should return No, not the default"
        );
    }

    #[test]
    fn garbage_input_returns_no() {
        let mut reader = Cursor::new(b"asdf\n");
        assert_eq!(
            ask_user("Q?", Answer::Yes, false, &mut reader).unwrap(),
            Answer::No,
            "garbage input should return No"
        );
    }

    #[test]
    fn n_with_escape_sequences_returns_no() {
        // Simulate terminal sending ANSI escape codes around the "n"
        let mut reader = Cursor::new(b"\x1b[?1ln\n");
        assert_eq!(
            ask_user("Q?", Answer::Yes, false, &mut reader).unwrap(),
            Answer::No,
            "should extract 'n' from input with escape sequences"
        );
    }

    #[test]
    fn n_with_control_chars_returns_no() {
        let mut reader = Cursor::new(b"\x00n\r\n");
        assert_eq!(
            ask_user("Q?", Answer::Yes, false, &mut reader).unwrap(),
            Answer::No,
        );
    }
}
