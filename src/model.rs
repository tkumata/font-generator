use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFont {
    pub name: String,
    pub sizes: Vec<GeneratedFontSize>,
    pub missing_glyphs: Vec<MissingGlyph>,
}

impl GeneratedFont {
    #[must_use]
    pub fn glyph_count(&self) -> usize {
        self.sizes.iter().map(|size| size.glyphs.len()).sum()
    }

    #[must_use]
    pub fn bitmap_bytes(&self) -> usize {
        self.sizes.iter().map(|size| size.bitmap_data.len()).sum()
    }

    #[must_use]
    pub fn format_summary(&self) -> String {
        let mut text = format!("font_model = {}\n", self.name);
        for size in &self.sizes {
            let _ = writeln!(
                text,
                "  - size {}: glyphs = {}, bitmap_bytes = {}",
                size.pixel_size,
                size.glyphs.len(),
                size.bitmap_data.len()
            );
        }
        let _ = writeln!(text, "missing_glyphs = {}", self.missing_glyphs.len());
        for missing in &self.missing_glyphs {
            let _ = writeln!(
                text,
                "  - {} ({})",
                format_display_unit(&missing.display_unit),
                missing.reason
            );
        }
        text
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFontSize {
    pub pixel_size: u32,
    pub glyphs: Vec<Glyph>,
    pub bitmap_data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Glyph {
    pub key: String,
    pub bitmap_offset: usize,
    pub bitmap_len: usize,
    pub width: u32,
    pub height: u32,
    pub advance_x: i32,
    pub bearing_x: i32,
    pub bearing_y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingGlyph {
    pub display_unit: String,
    pub reason: MissingGlyphReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingGlyphReason {
    NoFontGlyph,
    UnsupportedCluster,
}

impl std::fmt::Display for MissingGlyphReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoFontGlyph => formatter.write_str("no font glyph"),
            Self::UnsupportedCluster => formatter.write_str("unsupported grapheme cluster"),
        }
    }
}

fn format_display_unit(unit: &str) -> String {
    match unit {
        " " => "<space>".to_string(),
        _ => unit.to_string(),
    }
}
