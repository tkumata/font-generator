use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use unicode_segmentation::UnicodeSegmentation;

use crate::config::GenerationSettings;
use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterSet {
    display_units: Vec<String>,
}

impl CharacterSet {
    /// Reads configured character files and collects unique display units.
    ///
    /// # Errors
    ///
    /// Returns an error when a character file cannot be read as UTF-8 or no display units remain
    /// after whitespace filtering.
    pub fn from_settings(settings: &GenerationSettings) -> Result<Self, AppError> {
        collect_from_files(&settings.character_files, settings.preserve_space)
    }

    #[must_use]
    pub fn display_units(&self) -> &[String] {
        &self.display_units
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.display_units.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.display_units.is_empty()
    }

    #[must_use]
    pub fn format_summary(&self) -> String {
        let mut text = format!("characters = {}\n", self.display_units.len());
        for unit in &self.display_units {
            text.push_str("  - ");
            text.push_str(&format_display_unit(unit));
            text.push('\n');
        }
        text
    }
}

/// Collects unique display units from one or more character files.
///
/// # Errors
///
/// Returns an error when a file cannot be read as UTF-8 or no display units remain after
/// whitespace filtering.
pub fn collect_from_files(
    paths: &[PathBuf],
    preserve_space: bool,
) -> Result<CharacterSet, AppError> {
    let mut collector = CharacterCollector::new(preserve_space);
    for path in paths {
        collector.collect_file(path)?;
    }
    collector.finish()
}

struct CharacterCollector {
    preserve_space: bool,
    seen: HashSet<String>,
    display_units: Vec<String>,
}

impl CharacterCollector {
    fn new(preserve_space: bool) -> Self {
        Self {
            preserve_space,
            seen: HashSet::new(),
            display_units: Vec::new(),
        }
    }

    fn collect_file(&mut self, path: &Path) -> Result<(), AppError> {
        let content = fs::read_to_string(path).map_err(|source| AppError::CharacterFileRead {
            path: path.to_path_buf(),
            source,
        })?;
        self.collect_text(&content);
        Ok(())
    }

    fn collect_text(&mut self, content: &str) {
        for grapheme in UnicodeSegmentation::graphemes(content, true) {
            if should_ignore(grapheme, self.preserve_space) {
                continue;
            }

            if self.seen.insert(grapheme.to_string()) {
                self.display_units.push(grapheme.to_string());
            }
        }
    }

    fn finish(self) -> Result<CharacterSet, AppError> {
        if self.display_units.is_empty() {
            return Err(AppError::NoCharactersFound);
        }
        Ok(CharacterSet {
            display_units: self.display_units,
        })
    }
}

fn should_ignore(grapheme: &str, preserve_space: bool) -> bool {
    grapheme.chars().all(|ch| match ch {
        '\n' | '\r' | '\t' => true,
        ' ' => !preserve_space,
        _ => false,
    })
}

fn format_display_unit(unit: &str) -> String {
    match unit {
        " " => "<space>".to_string(),
        _ => unit.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::fs;

    use super::*;

    fn temp_file(
        test_name: &str,
        file_name: &str,
        content: &str,
    ) -> Result<PathBuf, std::io::Error> {
        let root = std::env::temp_dir().join(format!(
            "font-generator-chars-{test_name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;
        let path = root.join(file_name);
        fs::write(&path, content)?;
        Ok(path)
    }

    fn collect_text(content: &str, preserve_space: bool) -> Result<Vec<String>, AppError> {
        let mut collector = CharacterCollector::new(preserve_space);
        collector.collect_text(content);
        Ok(collector.finish()?.display_units)
    }

    fn ensure_eq<T>(actual: &T, expected: &T, field: &str) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Debug + PartialEq,
    {
        if actual == expected {
            Ok(())
        } else {
            Err(std::io::Error::other(format!(
                "{field} mismatch: expected {expected:?}, got {actual:?}"
            ))
            .into())
        }
    }

    fn ensure(condition: bool, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        if condition {
            Ok(())
        } else {
            Err(std::io::Error::other(message).into())
        }
    }

    fn ensure_units(
        actual: &[String],
        expected: &[&str],
        field: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let expected = expected.iter().map(ToString::to_string).collect::<Vec<_>>();
        ensure_eq(&actual.to_vec(), &expected, field)
    }

    #[test]
    fn removes_duplicate_ascii_in_stable_order() -> Result<(), Box<dyn std::error::Error>> {
        let units = collect_text("banana!", false)?;

        ensure_units(&units, &["b", "a", "n", "!"], "ascii units")?;
        Ok(())
    }

    #[test]
    fn collects_japanese_and_symbols() -> Result<(), Box<dyn std::error::Error>> {
        let units = collect_text("温度℃温", false)?;

        ensure_units(&units, &["温", "度", "℃"], "japanese units")?;
        Ok(())
    }

    #[test]
    fn keeps_emoji_variation_selector_with_base() -> Result<(), Box<dyn std::error::Error>> {
        let units = collect_text("☀️☀️☔️", false)?;

        ensure_units(&units, &["☀️", "☔️"], "emoji units")?;
        Ok(())
    }

    #[test]
    fn preserves_normal_space_when_configured() -> Result<(), Box<dyn std::error::Error>> {
        let units = collect_text("A B\nC", true)?;

        ensure_units(&units, &["A", " ", "B", "C"], "space units")?;
        Ok(())
    }

    #[test]
    fn ignores_normal_space_when_not_configured() -> Result<(), Box<dyn std::error::Error>> {
        let units = collect_text("A B\nC", false)?;

        ensure_units(&units, &["A", "B", "C"], "filtered units")?;
        Ok(())
    }

    #[test]
    fn reads_multiple_files_in_order() -> Result<(), Box<dyn std::error::Error>> {
        let first = temp_file("multiple", "first.txt", "abc")?;
        let second = first.with_file_name("second.txt");
        fs::write(&second, "cde")?;

        let characters = collect_from_files(&[first, second], false)?;

        ensure_units(
            characters.display_units(),
            &["a", "b", "c", "d", "e"],
            "multi-file units",
        )?;
        Ok(())
    }

    #[test]
    fn rejects_empty_character_set() -> Result<(), Box<dyn std::error::Error>> {
        let result = collect_text(" \n\t", false);

        ensure(
            matches!(result, Err(AppError::NoCharactersFound)),
            "empty character set should be rejected",
        )?;
        Ok(())
    }
}
