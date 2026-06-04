use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use crate::config::GenerationSettings;
use crate::error::AppError;
use crate::model::{GeneratedFont, GeneratedFontSize};
use crate::output::{byte_array_literal, c_string_literal};

pub fn write(
    settings: &GenerationSettings,
    font: &GeneratedFont,
) -> Result<Vec<PathBuf>, AppError> {
    let header_path = settings
        .output_directory
        .join(format!("{}.h", settings.output_name));
    let source_path = settings
        .output_directory
        .join(format!("{}.c", settings.output_name));

    fs::write(&header_path, render_header(&settings.output_name)).map_err(|source| {
        AppError::OutputWrite {
            path: header_path.clone(),
            source,
        }
    })?;
    fs::write(&source_path, render_source(&settings.output_name, font)).map_err(|source| {
        AppError::OutputWrite {
            path: source_path.clone(),
            source,
        }
    })?;

    Ok(vec![header_path, source_path])
}

fn render_header(name: &str) -> String {
    format!(
        r"#pragma once

#include <stddef.h>
#include <stdint.h>

typedef struct {{
    const char *key;
    uint32_t bitmap_offset;
    uint32_t bitmap_len;
    uint32_t width;
    uint32_t height;
    int32_t advance_x;
    int32_t bearing_x;
    int32_t bearing_y;
}} {name}_glyph_t;

typedef struct {{
    uint32_t pixel_size;
    const {name}_glyph_t *glyphs;
    size_t glyph_count;
    const uint8_t *bitmap_data;
    size_t bitmap_len;
}} {name}_font_size_t;

extern const {name}_font_size_t {name}_sizes[];
extern const size_t {name}_size_count;
"
    )
}

fn render_source(name: &str, font: &GeneratedFont) -> String {
    let mut text = format!("#include \"{name}.h\"\n\n");

    for size in &font.sizes {
        text.push_str(&render_size(name, size));
        text.push('\n');
    }

    let _ = writeln!(text, "const {name}_font_size_t {name}_sizes[] = {{");
    for size in &font.sizes {
        let suffix = size_suffix(size.pixel_size);
        let _ = writeln!(
            text,
            "    {{ {}u, {name}_{suffix}_glyphs, {}u, {name}_{suffix}_bitmap_data, {}u }},",
            size.pixel_size,
            size.glyphs.len(),
            size.bitmap_data.len()
        );
    }
    let _ = writeln!(text, "}};");
    let _ = writeln!(
        text,
        "const size_t {name}_size_count = {}u;",
        font.sizes.len()
    );

    text
}

fn render_size(name: &str, size: &GeneratedFontSize) -> String {
    let suffix = size_suffix(size.pixel_size);
    let mut text = String::new();
    let _ = writeln!(
        text,
        "static const uint8_t {name}_{suffix}_bitmap_data[] = {{"
    );
    if size.bitmap_data.is_empty() {
        let _ = writeln!(text, "    0x00,");
    } else {
        text.push_str(&byte_array_literal(&size.bitmap_data, "    "));
    }
    let _ = writeln!(text, "}};\n");

    let _ = writeln!(
        text,
        "static const {name}_glyph_t {name}_{suffix}_glyphs[] = {{"
    );
    if size.glyphs.is_empty() {
        let _ = writeln!(text, "    {{ \"\", 0u, 0u, 0u, 0u, 0, 0, 0 }},");
    } else {
        for glyph in &size.glyphs {
            let _ = writeln!(
                text,
                "    {{ {}, {}u, {}u, {}u, {}u, {}, {}, {} }},",
                c_string_literal(&glyph.key),
                glyph.bitmap_offset,
                glyph.bitmap_len,
                glyph.width,
                glyph.height,
                glyph.advance_x,
                glyph.bearing_x,
                glyph.bearing_y
            );
        }
    }
    let _ = writeln!(text, "}};");
    text
}

fn size_suffix(pixel_size: u32) -> String {
    format!("size_{pixel_size}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Glyph;

    #[test]
    fn renders_c_source_deterministically() -> Result<(), Box<dyn std::error::Error>> {
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

        let source = render_source("app_font", &font);

        if source.contains("static const uint8_t app_font_size_16_bitmap_data[]")
            && source.contains("{ \"A\", 0u, 2u, 2u, 2u, 3, 0, -1 }")
            && source.contains("const size_t app_font_size_count = 1u;")
        {
            Ok(())
        } else {
            Err(std::io::Error::other(source).into())
        }
    }
}
