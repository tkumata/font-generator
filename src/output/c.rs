use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use crate::alpha::pack_4bit_alpha;
use crate::config::{GenerationSettings, OutputFormat};
use crate::error::AppError;
use crate::model::{GeneratedFont, GeneratedFontSize, Glyph};
use crate::output::{byte_array_literal, c_string_literal};

pub fn write(
    settings: &GenerationSettings,
    font: &GeneratedFont,
) -> Result<Vec<PathBuf>, AppError> {
    if settings.output_format == Some(OutputFormat::CFixed) {
        return write_fixed(settings, font);
    }
    write_metrics(settings, font)
}

fn write_fixed(
    settings: &GenerationSettings,
    font: &GeneratedFont,
) -> Result<Vec<PathBuf>, AppError> {
    let header_path = settings
        .output_directory
        .join(format!("{}.h", settings.output_name));
    fs::write(&header_path, render_fixed_header(settings, font)?).map_err(|source| {
        AppError::OutputWrite {
            path: header_path.clone(),
            source,
        }
    })?;

    Ok(vec![header_path])
}

fn write_metrics(
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

fn render_fixed_header(
    settings: &GenerationSettings,
    font: &GeneratedFont,
) -> Result<String, AppError> {
    let cell = settings
        .fixed_cell
        .ok_or(AppError::MissingSetting("fixed_cell"))?;
    let size = font.sizes.first().ok_or(AppError::InvalidSetting {
        setting: "generation.sizes",
        message: "'c-fixed' requires exactly one generated size".to_string(),
    })?;
    let dimensions = FixedCellDimensions::from_config(cell.width, cell.height);
    let full_bytes_per_char = fixed_bytes_per_char(dimensions.full_width, dimensions.height)?;
    let half_bytes_per_char = fixed_bytes_per_char(dimensions.half_width, dimensions.height)?;
    let prefix = c_macro_prefix(&settings.output_name);
    let mut text = String::new();

    let _ = writeln!(
        text,
        "// Auto-generated 4-bit AA fixed-cell bitmap font: {}x{} full-width pixels",
        dimensions.full_width, dimensions.height
    );
    let _ = writeln!(
        text,
        "// Format: 4-bit grayscale alpha, nibble-packed, 2 pixels per byte"
    );
    let _ = writeln!(text, "// Pixel size: {}", size.pixel_size);
    let _ = writeln!(text);
    let _ = writeln!(text, "#ifndef {prefix}_H");
    let _ = writeln!(text, "#define {prefix}_H");
    let _ = writeln!(text);
    let _ = writeln!(text, "#include <stdint.h>");
    let _ = writeln!(text);
    let _ = writeln!(text, "#define {prefix}_WIDTH {}", dimensions.full_width);
    let _ = writeln!(text, "#define {prefix}_HEIGHT {}", dimensions.height);
    let _ = writeln!(
        text,
        "#define {prefix}_FULL_WIDTH {}",
        dimensions.full_width
    );
    let _ = writeln!(
        text,
        "#define {prefix}_HALF_WIDTH {}",
        dimensions.half_width
    );
    let _ = writeln!(text, "#define {prefix}_BPP 4");
    let _ = writeln!(
        text,
        "#define {prefix}_FULL_BYTES_PER_CHAR {full_bytes_per_char}"
    );
    let _ = writeln!(
        text,
        "#define {prefix}_HALF_BYTES_PER_CHAR {half_bytes_per_char}"
    );
    let _ = writeln!(
        text,
        "#define {prefix}_BYTES_PER_CHAR {full_bytes_per_char}"
    );
    let _ = writeln!(text, "#define {prefix}_CHAR_COUNT {}", size.glyphs.len());
    let _ = writeln!(text);
    let _ = writeln!(text, "static const char *{}_chars =", settings.output_name);
    let chunks = mapping_chunks(size, 32);
    for (index, chunk) in chunks.iter().enumerate() {
        let ending = if index + 1 == chunks.len() { ";" } else { "" };
        let _ = writeln!(text, "    {}{ending}", c_string_literal(chunk));
    }
    let _ = writeln!(text);
    let _ = writeln!(
        text,
        "static const uint8_t {}_widths[{}] = {{",
        settings.output_name,
        size.glyphs.len()
    );
    for glyph in &size.glyphs {
        let width = dimensions.width_for(&glyph.key);
        let _ = writeln!(text, "    {width}u, // '{}'", glyph.key);
    }
    let _ = writeln!(text, "}};");
    let _ = writeln!(text);
    let _ = writeln!(
        text,
        "static const uint8_t {}_data[{}][{}] = {{",
        settings.output_name,
        size.glyphs.len(),
        full_bytes_per_char
    );

    for glyph in &size.glyphs {
        let glyph_width = dimensions.width_for(&glyph.key);
        let packed = fixed_cell_bitmap(
            size,
            glyph,
            glyph_width,
            dimensions.height,
            full_bytes_per_char,
        )?;
        let _ = writeln!(text, "    {{// '{}'", glyph.key);
        text.push_str(&byte_array_literal(&packed, "     "));
        let _ = writeln!(text, "    }},");
    }

    let _ = writeln!(text, "}};");
    let _ = writeln!(text);
    let _ = writeln!(text, "#endif");

    Ok(text)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FixedCellDimensions {
    full_width: u32,
    half_width: u32,
    height: u32,
}

impl FixedCellDimensions {
    fn from_config(width: u32, height: u32) -> Self {
        Self {
            full_width: width - 1,
            half_width: (width / 2) - 1,
            height: height - 1,
        }
    }

    fn width_for(self, key: &str) -> u32 {
        if is_half_width_display_unit(key) {
            self.half_width
        } else {
            self.full_width
        }
    }
}

fn is_half_width_display_unit(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(character) = chars.next() else {
        return false;
    };
    chars.next().is_none() && (character == ' ' || character.is_ascii_graphic())
}

fn fixed_bytes_per_char(width: u32, height: u32) -> Result<usize, AppError> {
    let pixels = u64::from(width) * u64::from(height);
    let bytes = pixels.div_ceil(2);
    usize::try_from(bytes).map_err(|_| AppError::InvalidSetting {
        setting: "fixed_cell.width",
        message: "fixed cell byte count is too large for this host".to_string(),
    })
}

fn c_macro_prefix(name: &str) -> String {
    name.chars()
        .map(|ch| ch.to_ascii_uppercase())
        .collect::<String>()
}

fn mapping_chunks(size: &GeneratedFontSize, chunk_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut count = 0usize;

    for glyph in &size.glyphs {
        if count >= chunk_chars && !current.is_empty() {
            chunks.push(current);
            current = String::new();
            count = 0;
        }
        current.push_str(&glyph.key);
        count += 1;
    }

    if !current.is_empty() || chunks.is_empty() {
        chunks.push(current);
    }

    chunks
}

fn fixed_cell_bitmap(
    size: &GeneratedFontSize,
    glyph: &Glyph,
    target_width: u32,
    target_height: u32,
    row_bytes: usize,
) -> Result<Vec<u8>, AppError> {
    let cell_pixels = cell_pixel_count(target_width, target_height)?;
    let mut cell_alpha = vec![0u8; cell_pixels];
    let source_alpha = unpack_glyph_alpha(size, glyph)?;
    let source_width = usize::try_from(glyph.width).map_err(|_| AppError::MetricOverflow {
        glyph: glyph.key.clone(),
        metric: "width",
    })?;
    let source_height = usize::try_from(glyph.height).map_err(|_| AppError::MetricOverflow {
        glyph: glyph.key.clone(),
        metric: "height",
    })?;
    let target_width = usize::try_from(target_width).map_err(|_| AppError::InvalidSetting {
        setting: "fixed_cell.width",
        message: "fixed cell width cannot be represented on this host".to_string(),
    })?;
    let target_height = usize::try_from(target_height).map_err(|_| AppError::InvalidSetting {
        setting: "fixed_cell.height",
        message: "fixed cell height cannot be represented on this host".to_string(),
    })?;
    let copy_width = source_width.min(target_width);
    let copy_height = source_height.min(target_height);
    let skipped_source_columns = source_width.saturating_sub(target_width) / 2;
    let skipped_source_rows = source_height.saturating_sub(target_height) / 2;
    let target_column_offset = target_width.saturating_sub(source_width) / 2;
    let target_row_offset = target_height.saturating_sub(source_height) / 2;

    for row in 0..copy_height {
        for column in 0..copy_width {
            let Some(source_row) = skipped_source_rows.checked_add(row) else {
                return Err(AppError::InvalidSetting {
                    setting: "fixed_cell.height",
                    message: "fixed cell source row overflow".to_string(),
                });
            };
            let Some(source_column) = skipped_source_columns.checked_add(column) else {
                return Err(AppError::InvalidSetting {
                    setting: "fixed_cell.width",
                    message: "fixed cell source column overflow".to_string(),
                });
            };
            let Some(source_index) = source_row
                .checked_mul(source_width)
                .and_then(|base| base.checked_add(source_column))
            else {
                return Err(AppError::InvalidSetting {
                    setting: "fixed_cell.width",
                    message: "fixed cell source index overflow".to_string(),
                });
            };
            let Some(target_row) = target_row_offset.checked_add(row) else {
                return Err(AppError::InvalidSetting {
                    setting: "fixed_cell.height",
                    message: "fixed cell target row overflow".to_string(),
                });
            };
            let Some(target_column) = target_column_offset.checked_add(column) else {
                return Err(AppError::InvalidSetting {
                    setting: "fixed_cell.width",
                    message: "fixed cell target column overflow".to_string(),
                });
            };
            let Some(target_index) = target_row
                .checked_mul(target_width)
                .and_then(|base| base.checked_add(target_column))
            else {
                return Err(AppError::InvalidSetting {
                    setting: "fixed_cell.width",
                    message: "fixed cell target index overflow".to_string(),
                });
            };
            if let (Some(target), Some(source)) = (
                cell_alpha.get_mut(target_index),
                source_alpha.get(source_index),
            ) {
                *target = *source;
            }
        }
    }

    let mut packed = pack_4bit_alpha(&cell_alpha);
    if packed.len() > row_bytes {
        Err(AppError::InvalidSetting {
            setting: "fixed_cell.width",
            message: "fixed cell packed byte count exceeds row size".to_string(),
        })
    } else {
        packed.resize(row_bytes, 0);
        Ok(packed)
    }
}

fn cell_pixel_count(width: u32, height: u32) -> Result<usize, AppError> {
    let pixels = u64::from(width) * u64::from(height);
    usize::try_from(pixels).map_err(|_| AppError::InvalidSetting {
        setting: "fixed_cell.width",
        message: "fixed cell pixel count is too large for this host".to_string(),
    })
}

fn unpack_glyph_alpha(size: &GeneratedFontSize, glyph: &Glyph) -> Result<Vec<u8>, AppError> {
    let pixel_count = glyph_pixel_count(glyph)?;
    let end =
        glyph
            .bitmap_offset
            .checked_add(glyph.bitmap_len)
            .ok_or(AppError::InvalidSetting {
                setting: "generation.sizes",
                message: "glyph bitmap range overflow".to_string(),
            })?;
    let bytes = size
        .bitmap_data
        .get(glyph.bitmap_offset..end)
        .ok_or(AppError::InvalidSetting {
            setting: "generation.sizes",
            message: "glyph bitmap range is outside generated bitmap data".to_string(),
        })?;
    let mut alpha = Vec::with_capacity(pixel_count);

    for byte in bytes {
        if alpha.len() < pixel_count {
            alpha.push(byte >> 4);
        }
        if alpha.len() < pixel_count {
            alpha.push(byte & 0x0f);
        }
    }

    if alpha.len() == pixel_count {
        Ok(alpha)
    } else {
        Err(AppError::InvalidSetting {
            setting: "generation.sizes",
            message: "glyph bitmap data is shorter than glyph dimensions".to_string(),
        })
    }
}

fn glyph_pixel_count(glyph: &Glyph) -> Result<usize, AppError> {
    let pixels = u64::from(glyph.width) * u64::from(glyph.height);
    usize::try_from(pixels).map_err(|_| AppError::MetricOverflow {
        glyph: glyph.key.clone(),
        metric: "width * height",
    })
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
    fn derives_tight_fixed_cell_dimensions() {
        let dimensions = FixedCellDimensions::from_config(26, 26);

        assert_eq!(
            dimensions,
            FixedCellDimensions {
                full_width: 25,
                half_width: 12,
                height: 25,
            }
        );
    }

    #[test]
    fn classifies_ascii_display_units_as_half_width() {
        assert!(is_half_width_display_unit("A"));
        assert!(is_half_width_display_unit("7"));
        assert!(is_half_width_display_unit("!"));
        assert!(is_half_width_display_unit(" "));
        assert!(!is_half_width_display_unit("あ"));
        assert!(!is_half_width_display_unit("AB"));
    }

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
