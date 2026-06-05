use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("config file not found: {0}")]
    ConfigNotFound(PathBuf),
    #[error("failed to read config file {path}: {source}")]
    ConfigRead {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse config file {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("missing required setting: {0}")]
    MissingSetting(&'static str),
    #[error("invalid setting {setting}: {message}")]
    InvalidSetting {
        setting: &'static str,
        message: String,
    },
    #[error("font file not found: {0}")]
    FontNotFound(PathBuf),
    #[error("failed to read font file {path}: {source}")]
    FontRead {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse font file {path}: {message}")]
    FontParse { path: PathBuf, message: String },
    #[error("character file not found: {0}")]
    CharacterFileNotFound(PathBuf),
    #[error("failed to read character file {path}: {source}")]
    CharacterFileRead {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("no characters found in character input files")]
    NoCharactersFound,
    #[error("missing glyphs: {0}")]
    MissingGlyphs(String),
    #[error("metric overflow for glyph {glyph}: {metric}")]
    MetricOverflow { glyph: String, metric: &'static str },
    #[error(
        "glyph {glyph} bitmap {glyph_width}x{glyph_height} does not fit fixed cell {cell_width}x{cell_height}"
    )]
    FixedCellGlyphTooLarge {
        glyph: String,
        glyph_width: u32,
        glyph_height: u32,
        cell_width: u32,
        cell_height: u32,
    },
    #[error("output directory not found: {0}")]
    OutputDirectoryNotFound(PathBuf),
    #[error("failed to write output file {path}: {source}")]
    OutputWrite {
        path: PathBuf,
        source: std::io::Error,
    },
}
