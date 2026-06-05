use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::cli::{Cli, LanguageArg};
use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputLanguage {
    C,
    Rust,
}

impl OutputLanguage {
    fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "c" => Ok(Self::C),
            "rust" => Ok(Self::Rust),
            _ => Err(AppError::InvalidSetting {
                setting: "output.language",
                message: "expected 'c' or 'rust'".to_string(),
            }),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::C => "c",
            Self::Rust => "rust",
        }
    }
}

impl From<LanguageArg> for OutputLanguage {
    fn from(value: LanguageArg) -> Self {
        match value {
            LanguageArg::C => Self::C,
            LanguageArg::Rust => Self::Rust,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    CFixed,
    CMetrics,
}

impl OutputFormat {
    fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "c-fixed" => Ok(Self::CFixed),
            "c-metrics" => Ok(Self::CMetrics),
            _ => Err(AppError::InvalidSetting {
                setting: "output.format",
                message: "expected 'c-fixed' or 'c-metrics'".to_string(),
            }),
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CFixed => "c-fixed",
            Self::CMetrics => "c-metrics",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedCellSettings {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationSettings {
    pub config_path: Option<PathBuf>,
    pub font_path: PathBuf,
    pub character_files: Vec<PathBuf>,
    pub preserve_space: bool,
    pub language: OutputLanguage,
    pub output_format: Option<OutputFormat>,
    pub output_name: String,
    pub output_directory: PathBuf,
    pub sizes: Vec<u32>,
    pub alpha_bits: u8,
    pub missing_glyphs: MissingGlyphPolicy,
    pub fixed_cell: Option<FixedCellSettings>,
}

impl GenerationSettings {
    /// Builds validated generation settings from CLI arguments and optional TOML config.
    ///
    /// # Errors
    ///
    /// Returns an error when the config file cannot be read, required settings are missing,
    /// configured paths do not exist, or a setting value is unsupported.
    pub fn from_cli(cli: &Cli) -> Result<Self, AppError> {
        let config_path = if let Some(path) = &cli.config {
            Some(normalize_existing_config_path(path)?)
        } else {
            let default_path = PathBuf::from("fontgen.toml");
            if default_path.is_file() {
                Some(normalize_existing_config_path(&default_path)?)
            } else {
                None
            }
        };
        let raw = match &config_path {
            Some(path) => RawConfig::load(path)?,
            None => RawConfig::default(),
        };
        Self::from_raw_and_cli(config_path, raw, cli)
    }

    #[must_use]
    pub fn format_normalized(&self) -> String {
        let mut text = String::new();
        match &self.config_path {
            Some(path) => {
                let _ = writeln!(text, "config = {}", path.display());
            }
            None => {
                let _ = writeln!(text, "config = <none>");
            }
        }
        let _ = writeln!(text, "font = {}", self.font_path.display());
        let _ = writeln!(text, "chars =");
        for path in &self.character_files {
            let _ = writeln!(text, "  - {}", path.display());
        }
        let _ = writeln!(text, "preserve_space = {}", self.preserve_space);
        let _ = writeln!(text, "language = {}", self.language.as_str());
        if let Some(output_format) = self.output_format {
            let _ = writeln!(text, "output_format = {}", output_format.as_str());
        } else {
            let _ = writeln!(text, "output_format = <default>");
        }
        let _ = writeln!(text, "output_name = {}", self.output_name);
        let _ = writeln!(
            text,
            "output_directory = {}",
            self.output_directory.display()
        );
        let _ = writeln!(text, "sizes = {:?}", self.sizes);
        let _ = writeln!(text, "alpha_bits = {}", self.alpha_bits);
        let _ = writeln!(text, "missing_glyphs = {}", self.missing_glyphs.as_str());
        if let Some(fixed_cell) = self.fixed_cell {
            let _ = writeln!(
                text,
                "fixed_cell = {}x{}",
                fixed_cell.width, fixed_cell.height
            );
        } else {
            let _ = writeln!(text, "fixed_cell = <none>");
        }
        text
    }

    fn from_raw_and_cli(
        config_path: Option<PathBuf>,
        raw: RawConfig,
        cli: &Cli,
    ) -> Result<Self, AppError> {
        let config_dir = config_path
            .as_deref()
            .and_then(Path::parent)
            .unwrap_or_else(|| Path::new("."));

        let font_path = cli
            .font
            .clone()
            .or(raw.font.and_then(|font| font.path))
            .ok_or(AppError::MissingSetting("font.path"))?;
        let font_path = resolve_path(config_dir, font_path);

        let character_files = if cli.chars.is_empty() {
            raw.input
                .as_ref()
                .and_then(|input| input.chars.clone())
                .ok_or(AppError::MissingSetting("input.chars"))?
        } else {
            cli.chars.clone()
        };

        if character_files.is_empty() {
            return Err(AppError::InvalidSetting {
                setting: "input.chars",
                message: "at least one character file is required".to_string(),
            });
        }

        let character_files = character_files
            .into_iter()
            .map(|path| resolve_path(config_dir, path))
            .collect::<Vec<_>>();

        let output = raw.output.unwrap_or_default();
        let generation = raw.generation.unwrap_or_default();

        let preserve_space = cli
            .preserve_space
            .or_else(|| raw.input.as_ref().and_then(|input| input.preserve_space))
            .unwrap_or(false);

        let language = match cli.language {
            Some(language) => OutputLanguage::from(language),
            None => output
                .language
                .as_deref()
                .map(OutputLanguage::parse)
                .transpose()?
                .unwrap_or(OutputLanguage::C),
        };

        let output_format = output
            .format
            .as_deref()
            .map(OutputFormat::parse)
            .transpose()?;

        let output_name = cli
            .output_name
            .clone()
            .or(output.name)
            .unwrap_or_else(|| "app_font".to_string());

        validate_symbol_name(&output_name)?;

        let output_directory = cli
            .output_dir
            .clone()
            .or(output.directory)
            .unwrap_or_else(|| PathBuf::from("."));
        let output_directory = resolve_path(config_dir, output_directory);

        let sizes = if cli.sizes.is_empty() {
            generation
                .sizes
                .ok_or(AppError::MissingSetting("generation.sizes"))?
        } else {
            cli.sizes.clone()
        };

        validate_sizes(&sizes)?;

        let alpha_bits = generation.alpha_bits.unwrap_or(4);
        if alpha_bits != 4 {
            return Err(AppError::InvalidSetting {
                setting: "generation.alpha_bits",
                message: "phase 2 supports only 4-bit alpha".to_string(),
            });
        }

        let fixed_cell = parse_fixed_cell(raw.fixed_cell)?;

        let missing_glyphs = generation
            .missing_glyphs
            .as_deref()
            .map(MissingGlyphPolicy::parse)
            .transpose()?
            .unwrap_or(MissingGlyphPolicy::Error);

        validate_output_format(language, output_format, fixed_cell, &sizes)?;

        let settings = Self {
            config_path,
            font_path,
            character_files,
            preserve_space,
            language,
            output_format,
            output_name,
            output_directory,
            sizes,
            alpha_bits,
            missing_glyphs,
            fixed_cell,
        };

        settings.validate_paths()?;
        Ok(settings)
    }

    fn validate_paths(&self) -> Result<(), AppError> {
        if !self.font_path.is_file() {
            return Err(AppError::FontNotFound(self.font_path.clone()));
        }
        for path in &self.character_files {
            if !path.is_file() {
                return Err(AppError::CharacterFileNotFound(path.clone()));
            }
        }
        if !self.output_directory.is_dir() {
            return Err(AppError::OutputDirectoryNotFound(
                self.output_directory.clone(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingGlyphPolicy {
    Error,
    Skip,
}

impl MissingGlyphPolicy {
    fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "error" => Ok(Self::Error),
            "skip" => Ok(Self::Skip),
            _ => Err(AppError::InvalidSetting {
                setting: "generation.missing_glyphs",
                message: "expected 'error' or 'skip'".to_string(),
            }),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Skip => "skip",
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawConfig {
    font: Option<FontConfig>,
    input: Option<InputConfig>,
    output: Option<OutputConfig>,
    generation: Option<GenerationConfig>,
    fixed_cell: Option<FixedCellConfig>,
}

impl RawConfig {
    fn load(path: &Path) -> Result<Self, AppError> {
        let content = fs::read_to_string(path).map_err(|source| AppError::ConfigRead {
            path: path.to_path_buf(),
            source,
        })?;
        toml::from_str(&content).map_err(|source| AppError::ConfigParse {
            path: path.to_path_buf(),
            source,
        })
    }
}

#[derive(Debug, Deserialize)]
struct FontConfig {
    path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct InputConfig {
    chars: Option<Vec<PathBuf>>,
    preserve_space: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
struct OutputConfig {
    language: Option<String>,
    name: Option<String>,
    directory: Option<PathBuf>,
    format: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct GenerationConfig {
    sizes: Option<Vec<u32>>,
    alpha_bits: Option<u8>,
    missing_glyphs: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FixedCellConfig {
    width: Option<u32>,
    height: Option<u32>,
}

fn parse_fixed_cell(
    fixed_cell: Option<FixedCellConfig>,
) -> Result<Option<FixedCellSettings>, AppError> {
    fixed_cell
        .map(|cell| {
            let width = cell
                .width
                .ok_or(AppError::MissingSetting("fixed_cell.width"))?;
            let height = cell
                .height
                .ok_or(AppError::MissingSetting("fixed_cell.height"))?;
            validate_fixed_cell(width, height)?;
            Ok(FixedCellSettings { width, height })
        })
        .transpose()
}

fn normalize_existing_config_path(path: &Path) -> Result<PathBuf, AppError> {
    if !path.is_file() {
        return Err(AppError::ConfigNotFound(path.to_path_buf()));
    }
    path.canonicalize().map_err(|source| AppError::ConfigRead {
        path: path.to_path_buf(),
        source,
    })
}

fn resolve_path(config_dir: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        config_dir.join(path)
    }
}

fn validate_sizes(sizes: &[u32]) -> Result<(), AppError> {
    if sizes.is_empty() {
        return Err(AppError::InvalidSetting {
            setting: "generation.sizes",
            message: "at least one size is required".to_string(),
        });
    }
    if sizes.contains(&0) {
        return Err(AppError::InvalidSetting {
            setting: "generation.sizes",
            message: "sizes must be positive pixel values".to_string(),
        });
    }
    Ok(())
}

fn validate_fixed_cell(width: u32, height: u32) -> Result<(), AppError> {
    if width == 0 {
        return Err(AppError::InvalidSetting {
            setting: "fixed_cell.width",
            message: "width must be a positive pixel value".to_string(),
        });
    }
    if height == 0 {
        return Err(AppError::InvalidSetting {
            setting: "fixed_cell.height",
            message: "height must be a positive pixel value".to_string(),
        });
    }
    Ok(())
}

fn validate_output_format(
    language: OutputLanguage,
    output_format: Option<OutputFormat>,
    fixed_cell: Option<FixedCellSettings>,
    sizes: &[u32],
) -> Result<(), AppError> {
    if output_format != Some(OutputFormat::CFixed) {
        return Ok(());
    }
    if language != OutputLanguage::C {
        return Err(AppError::InvalidSetting {
            setting: "output.format",
            message: "'c-fixed' requires output.language = 'c'".to_string(),
        });
    }
    if fixed_cell.is_none() {
        return Err(AppError::MissingSetting("fixed_cell"));
    }
    if sizes.len() != 1 {
        return Err(AppError::InvalidSetting {
            setting: "generation.sizes",
            message: "'c-fixed' requires exactly one size".to_string(),
        });
    }
    Ok(())
}

fn validate_symbol_name(name: &str) -> Result<(), AppError> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(AppError::InvalidSetting {
            setting: "output.name",
            message: "name cannot be empty".to_string(),
        });
    };

    if !(first == '_' || first.is_ascii_alphabetic()) {
        return Err(AppError::InvalidSetting {
            setting: "output.name",
            message: "name must start with an ASCII letter or underscore".to_string(),
        });
    }

    if chars.any(|ch| !(ch == '_' || ch.is_ascii_alphanumeric())) {
        return Err(AppError::InvalidSetting {
            setting: "output.name",
            message: "name must contain only ASCII letters, digits, or underscores".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;

    fn temp_project(test_name: &str) -> Result<PathBuf, std::io::Error> {
        let root =
            std::env::temp_dir().join(format!("font-generator-{test_name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("fonts"))?;
        fs::create_dir_all(root.join("chars"))?;
        fs::write(root.join("fonts/font.ttf"), b"font")?;
        fs::write(root.join("chars/main.txt"), "abc")?;
        root.canonicalize()
    }

    fn write_config(root: &Path, body: &str) -> Result<PathBuf, std::io::Error> {
        let path = root.join("fontgen.toml");
        fs::write(&path, body)?;
        Ok(path)
    }

    fn cli_for(config: PathBuf) -> Cli {
        Cli {
            config: Some(config),
            font: None,
            chars: Vec::new(),
            sizes: Vec::new(),
            language: None,
            output_name: None,
            output_dir: None,
            preserve_space: None,
        }
    }

    fn cli_without_config(root: &Path) -> Cli {
        Cli {
            config: None,
            font: Some(root.join("fonts/font.ttf")),
            chars: vec![root.join("chars/main.txt")],
            sizes: vec![14],
            language: Some(LanguageArg::C),
            output_name: Some("cli_font".to_string()),
            output_dir: Some(root.to_path_buf()),
            preserve_space: Some(false),
        }
    }

    fn ensure_eq<T>(field: &str, actual: &T, expected: &T) -> Result<(), Box<dyn std::error::Error>>
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

    #[test]
    fn parses_config_into_normalized_settings() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_project("parse")?;
        let config = write_config(
            &root,
            r#"
                [font]
                path = "fonts/font.ttf"

                [input]
                chars = ["chars/main.txt"]
                preserve_space = true

                [output]
                language = "rust"
                name = "weather_font"
                directory = "."

                [generation]
                sizes = [16, 24]
                alpha_bits = 4
                missing_glyphs = "skip"
            "#,
        )?;

        let settings = GenerationSettings::from_cli(&cli_for(config))?;

        ensure_eq(
            "config_path",
            &settings.config_path,
            &Some(root.join("fontgen.toml")),
        )?;
        ensure_eq(
            "font_path",
            &settings.font_path,
            &root.join("fonts/font.ttf"),
        )?;
        ensure_eq(
            "character_files",
            &settings.character_files,
            &vec![root.join("chars/main.txt")],
        )?;
        ensure(settings.preserve_space, "preserve_space should be true")?;
        ensure_eq("language", &settings.language, &OutputLanguage::Rust)?;
        ensure_eq("output_format", &settings.output_format, &None)?;
        ensure_eq(
            "output_name",
            &settings.output_name,
            &"weather_font".to_string(),
        )?;
        ensure_eq("output_directory", &settings.output_directory, &root)?;
        ensure_eq("sizes", &settings.sizes, &vec![16, 24])?;
        ensure_eq("alpha_bits", &settings.alpha_bits, &4)?;
        ensure_eq("fixed_cell", &settings.fixed_cell, &None)?;
        ensure_eq(
            "missing_glyphs",
            &settings.missing_glyphs,
            &MissingGlyphPolicy::Skip,
        )?;
        Ok(())
    }

    #[test]
    fn accepts_cli_only_settings_when_no_config_exists() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_project("cli-only")?;

        let result = GenerationSettings::from_cli(&cli_without_config(&root));

        let settings = result?;
        ensure_eq("config_path", &settings.config_path, &None)?;
        ensure_eq(
            "font_path",
            &settings.font_path,
            &root.join("fonts/font.ttf"),
        )?;
        ensure_eq(
            "character_files",
            &settings.character_files,
            &vec![root.join("chars/main.txt")],
        )?;
        ensure_eq(
            "output_name",
            &settings.output_name,
            &"cli_font".to_string(),
        )?;
        ensure_eq("sizes", &settings.sizes, &vec![14])?;
        ensure_eq("output_format", &settings.output_format, &None)?;
        ensure_eq("fixed_cell", &settings.fixed_cell, &None)?;
        Ok(())
    }

    #[test]
    fn parses_fixed_cell_c_output_settings() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_project("fixed-cell")?;
        let config = write_config(
            &root,
            r#"
                [font]
                path = "fonts/font.ttf"

                [input]
                chars = ["chars/main.txt"]

                [output]
                language = "c"
                name = "fixed_font"
                directory = "."
                format = "c-fixed"

                [generation]
                sizes = [26]

                [fixed_cell]
                width = 26
                height = 26
            "#,
        )?;

        let settings = GenerationSettings::from_cli(&cli_for(config))?;

        ensure_eq("language", &settings.language, &OutputLanguage::C)?;
        ensure_eq(
            "output_format",
            &settings.output_format,
            &Some(OutputFormat::CFixed),
        )?;
        ensure_eq(
            "fixed_cell",
            &settings.fixed_cell,
            &Some(FixedCellSettings {
                width: 26,
                height: 26,
            }),
        )?;
        ensure_eq("sizes", &settings.sizes, &vec![26])?;
        Ok(())
    }

    #[test]
    fn rejects_fixed_cell_c_output_with_multiple_sizes() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_project("fixed-cell-multiple-sizes")?;
        let config = write_config(
            &root,
            r#"
                [font]
                path = "fonts/font.ttf"

                [input]
                chars = ["chars/main.txt"]

                [output]
                language = "c"
                format = "c-fixed"

                [generation]
                sizes = [16, 26]

                [fixed_cell]
                width = 26
                height = 26
            "#,
        )?;

        ensure(
            matches!(
                GenerationSettings::from_cli(&cli_for(config)),
                Err(AppError::InvalidSetting {
                    setting: "generation.sizes",
                    ..
                })
            ),
            "expected generation.sizes invalid setting error",
        )?;
        Ok(())
    }

    #[test]
    fn rejects_fixed_cell_c_output_without_cell_settings() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = temp_project("fixed-cell-missing")?;
        let config = write_config(
            &root,
            r#"
                [font]
                path = "fonts/font.ttf"

                [input]
                chars = ["chars/main.txt"]

                [output]
                language = "c"
                format = "c-fixed"

                [generation]
                sizes = [26]
            "#,
        )?;

        ensure(
            matches!(
                GenerationSettings::from_cli(&cli_for(config)),
                Err(AppError::MissingSetting("fixed_cell"))
            ),
            "expected missing fixed_cell setting",
        )?;
        Ok(())
    }

    #[test]
    fn cli_overrides_replace_config_values() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_project("overrides")?;
        fs::write(root.join("fonts/override.otf"), b"font")?;
        fs::write(root.join("chars/override.txt"), "xyz")?;
        let config = write_config(
            &root,
            r#"
                [font]
                path = "fonts/font.ttf"

                [input]
                chars = ["chars/main.txt"]
                preserve_space = false

                [output]
                language = "c"
                name = "base_font"
                directory = "."

                [generation]
                sizes = [12]
            "#,
        )?;

        let cli = Cli {
            config: Some(config),
            font: Some(PathBuf::from("fonts/override.otf")),
            chars: vec![PathBuf::from("chars/override.txt")],
            sizes: vec![18, 20],
            language: Some(LanguageArg::Rust),
            output_name: Some("override_font".to_string()),
            output_dir: Some(PathBuf::from(".")),
            preserve_space: Some(true),
        };

        let settings = GenerationSettings::from_cli(&cli)?;

        ensure_eq(
            "font_path",
            &settings.font_path,
            &root.join("fonts/override.otf"),
        )?;
        ensure_eq(
            "character_files",
            &settings.character_files,
            &vec![root.join("chars/override.txt")],
        )?;
        ensure_eq("sizes", &settings.sizes, &vec![18, 20])?;
        ensure_eq("language", &settings.language, &OutputLanguage::Rust)?;
        ensure_eq(
            "output_name",
            &settings.output_name,
            &"override_font".to_string(),
        )?;
        ensure(settings.preserve_space, "preserve_space should be true")?;
        Ok(())
    }

    #[test]
    fn rejects_missing_character_files() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_project("missing-chars")?;
        let config = write_config(
            &root,
            r#"
                [font]
                path = "fonts/font.ttf"

                [input]
                chars = ["chars/missing.txt"]

                [generation]
                sizes = [16]
            "#,
        )?;

        ensure(
            matches!(
                GenerationSettings::from_cli(&cli_for(config)),
                Err(AppError::CharacterFileNotFound(_))
            ),
            "expected missing character file error",
        )?;
        Ok(())
    }

    #[test]
    fn rejects_zero_sizes() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_project("zero-size")?;
        let config = write_config(
            &root,
            r#"
                [font]
                path = "fonts/font.ttf"

                [input]
                chars = ["chars/main.txt"]

                [generation]
                sizes = [0]
            "#,
        )?;

        ensure(
            matches!(
                GenerationSettings::from_cli(&cli_for(config)),
                Err(AppError::InvalidSetting {
                    setting: "generation.sizes",
                    ..
                })
            ),
            "expected generation.sizes invalid setting error",
        )?;
        Ok(())
    }
}
