use std::fmt::Write as _;
use std::path::PathBuf;

use crate::config::{GenerationSettings, OutputLanguage};
use crate::error::AppError;
use crate::model::GeneratedFont;

mod c;
mod rust;

/// Writes generated font output files for the configured language.
///
/// # Errors
///
/// Returns an error when an output file cannot be written.
pub fn write_output(
    settings: &GenerationSettings,
    font: &GeneratedFont,
) -> Result<Vec<PathBuf>, AppError> {
    match settings.language {
        OutputLanguage::C => c::write(settings, font),
        OutputLanguage::Rust => rust::write(settings, font),
    }
}

#[must_use]
pub fn format_written_files(paths: &[PathBuf]) -> String {
    let mut text = format!("output_files = {}\n", paths.len());
    for path in paths {
        let _ = writeln!(text, "  - {}", path.display());
    }
    text
}

fn c_string_literal(value: &str) -> String {
    let mut escaped = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped.push('"');
    escaped
}

fn rust_string_literal(value: &str) -> String {
    format!("{value:?}")
}

fn byte_array_literal(bytes: &[u8], indent: &str) -> String {
    let mut text = String::new();
    for chunk in bytes.chunks(12) {
        text.push_str(indent);
        for byte in chunk {
            let _ = write!(text, "0x{byte:02x}, ");
        }
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_c_string_literals() -> Result<(), Box<dyn std::error::Error>> {
        let escaped = c_string_literal("A\"\\\n");

        if escaped == "\"A\\\"\\\\\\n\"" {
            Ok(())
        } else {
            Err(std::io::Error::other(format!("unexpected C escape: {escaped}")).into())
        }
    }
}
