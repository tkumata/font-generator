use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use crate::config::GenerationSettings;
use crate::error::AppError;
use crate::model::{GeneratedFont, GeneratedFontSize};
use crate::output::{byte_array_literal, rust_string_literal};

pub fn write(
    settings: &GenerationSettings,
    font: &GeneratedFont,
) -> Result<Vec<PathBuf>, AppError> {
    let path = settings
        .output_directory
        .join(format!("{}.rs", settings.output_name));
    fs::write(&path, render_module(font)).map_err(|source| AppError::OutputWrite {
        path: path.clone(),
        source,
    })?;
    Ok(vec![path])
}

fn render_module(font: &GeneratedFont) -> String {
    let mut text = String::new();
    text.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
    text.push_str("pub struct Glyph {\n");
    text.push_str("    pub key: &'static str,\n");
    text.push_str("    pub bitmap_offset: usize,\n");
    text.push_str("    pub bitmap_len: usize,\n");
    text.push_str("    pub width: u32,\n");
    text.push_str("    pub height: u32,\n");
    text.push_str("    pub advance_x: i32,\n");
    text.push_str("    pub bearing_x: i32,\n");
    text.push_str("    pub bearing_y: i32,\n");
    text.push_str("}\n\n");
    text.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
    text.push_str("pub struct FontSize {\n");
    text.push_str("    pub pixel_size: u32,\n");
    text.push_str("    pub glyphs: &'static [Glyph],\n");
    text.push_str("    pub bitmap_data: &'static [u8],\n");
    text.push_str("}\n\n");

    for size in &font.sizes {
        text.push_str(&render_size(size));
        text.push('\n');
    }

    let _ = writeln!(text, "pub const FONT_SIZES: &[FontSize] = &[");
    for size in &font.sizes {
        let suffix = size_suffix(size.pixel_size);
        let _ = writeln!(
            text,
            "    FontSize {{ pixel_size: {}u32, glyphs: {suffix}_GLYPHS, bitmap_data: {suffix}_BITMAP_DATA }},",
            size.pixel_size
        );
    }
    let _ = writeln!(text, "];");

    text
}

fn render_size(size: &GeneratedFontSize) -> String {
    let suffix = size_suffix(size.pixel_size);
    let mut text = String::new();

    let _ = writeln!(text, "pub const {suffix}_BITMAP_DATA: &[u8] = &[");
    text.push_str(&byte_array_literal(&size.bitmap_data, "    "));
    let _ = writeln!(text, "];\n");

    let _ = writeln!(text, "pub const {suffix}_GLYPHS: &[Glyph] = &[");
    for glyph in &size.glyphs {
        let _ = writeln!(
            text,
            "    Glyph {{ key: {}, bitmap_offset: {}, bitmap_len: {}, width: {}u32, height: {}u32, advance_x: {}, bearing_x: {}, bearing_y: {} }},",
            rust_string_literal(&glyph.key),
            glyph.bitmap_offset,
            glyph.bitmap_len,
            glyph.width,
            glyph.height,
            glyph.advance_x,
            glyph.bearing_x,
            glyph.bearing_y
        );
    }
    let _ = writeln!(text, "];");
    text
}

fn size_suffix(pixel_size: u32) -> String {
    format!("SIZE_{pixel_size}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Glyph;

    #[test]
    fn renders_rust_module_deterministically() -> Result<(), Box<dyn std::error::Error>> {
        let font = GeneratedFont {
            name: "app_font".to_string(),
            sizes: vec![GeneratedFontSize {
                pixel_size: 16,
                glyphs: vec![Glyph {
                    key: "A".to_string(),
                    bitmap_offset: 0,
                    bitmap_len: 2,
                    width: 2,
                    height: 2,
                    advance_x: 3,
                    bearing_x: 0,
                    bearing_y: -1,
                }],
                bitmap_data: vec![0xf0, 0x81],
            }],
            missing_glyphs: Vec::new(),
        };

        let module = render_module(&font);

        if module.contains("pub const SIZE_16_BITMAP_DATA: &[u8]")
            && module.contains("Glyph { key: \"A\", bitmap_offset: 0, bitmap_len: 2")
            && module.contains("pub const FONT_SIZES: &[FontSize]")
        {
            Ok(())
        } else {
            Err(std::io::Error::other(module).into())
        }
    }
}
