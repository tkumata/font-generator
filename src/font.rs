use std::fs;
use std::path::{Path, PathBuf};

use fontdue::{Font, FontSettings};

use crate::alpha::{pack_4bit_alpha, quantize_bitmap};
use crate::chars::CharacterSet;
use crate::config::{GenerationSettings, MissingGlyphPolicy};
use crate::error::AppError;
use crate::model::{GeneratedFont, GeneratedFontSize, Glyph, MissingGlyph, MissingGlyphReason};

/// Builds the in-memory generated font model for configured characters and sizes.
///
/// # Errors
///
/// Returns an error when the font cannot be read or parsed, a requested glyph is missing while
/// using the `error` missing-glyph policy, or a metric value cannot be represented in the model.
pub fn generate_font_model(
    settings: &GenerationSettings,
    characters: &CharacterSet,
) -> Result<GeneratedFont, AppError> {
    let rasterizer = FontRasterizer::load(&settings.font_path)?;
    rasterizer.rasterize(settings, characters)
}

struct FontRasterizer {
    font: Font,
}

impl FontRasterizer {
    fn load(path: &Path) -> Result<Self, AppError> {
        let data = fs::read(path).map_err(|source| AppError::FontRead {
            path: path.to_path_buf(),
            source,
        })?;
        let font = Font::from_bytes(data, FontSettings::default()).map_err(|message| {
            AppError::FontParse {
                path: PathBuf::from(path),
                message: message.to_string(),
            }
        })?;
        Ok(Self { font })
    }

    fn rasterize(
        &self,
        settings: &GenerationSettings,
        characters: &CharacterSet,
    ) -> Result<GeneratedFont, AppError> {
        let mut missing_glyphs = Vec::new();
        let mut renderable_units = Vec::new();

        for unit in characters.display_units() {
            match self.resolve_char(unit) {
                Ok(character) => renderable_units.push((unit.as_str(), character)),
                Err(reason) => {
                    missing_glyphs.push(MissingGlyph {
                        display_unit: unit.clone(),
                        reason,
                    });
                }
            }
        }

        if settings.missing_glyphs == MissingGlyphPolicy::Error && !missing_glyphs.is_empty() {
            return Err(AppError::MissingGlyphs(format_missing_glyphs(
                &missing_glyphs,
            )));
        }

        let sizes = settings
            .sizes
            .iter()
            .copied()
            .map(|size| self.rasterize_size(size, &renderable_units))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(GeneratedFont {
            name: settings.output_name.clone(),
            sizes,
            missing_glyphs,
        })
    }

    fn resolve_char(&self, unit: &str) -> Result<char, MissingGlyphReason> {
        let mut chars = unit.chars();
        let Some(character) = chars.next() else {
            return Err(MissingGlyphReason::UnsupportedCluster);
        };
        if chars.next().is_some() {
            return Err(MissingGlyphReason::UnsupportedCluster);
        }
        if self.font.has_glyph(character) {
            Ok(character)
        } else {
            Err(MissingGlyphReason::NoFontGlyph)
        }
    }

    fn rasterize_size(
        &self,
        pixel_size: u32,
        units: &[(&str, char)],
    ) -> Result<GeneratedFontSize, AppError> {
        let mut glyphs = Vec::with_capacity(units.len());
        let mut bitmap_data = Vec::new();
        let px = pixel_size_to_f32(pixel_size)?;

        for (key, character) in units {
            let (metrics, coverage) = self.font.rasterize(*character, px);
            let bitmap_offset = bitmap_data.len();
            let alpha = quantize_bitmap(&coverage);
            let packed = pack_4bit_alpha(&alpha);
            let bitmap_len = packed.len();
            bitmap_data.extend(packed);

            glyphs.push(Glyph {
                key: (*key).to_string(),
                bitmap_offset,
                bitmap_len,
                width: u32::try_from(metrics.width).map_err(|_| AppError::MetricOverflow {
                    glyph: (*key).to_string(),
                    metric: "width",
                })?,
                height: u32::try_from(metrics.height).map_err(|_| AppError::MetricOverflow {
                    glyph: (*key).to_string(),
                    metric: "height",
                })?,
                advance_x: round_metric((*key).to_string(), "advance_x", metrics.advance_width)?,
                bearing_x: metrics.xmin,
                bearing_y: metrics.ymin,
            });
        }

        Ok(GeneratedFontSize {
            pixel_size,
            glyphs,
            bitmap_data,
        })
    }
}

fn pixel_size_to_f32(pixel_size: u32) -> Result<f32, AppError> {
    let size = u16::try_from(pixel_size).map_err(|_| AppError::InvalidSetting {
        setting: "generation.sizes",
        message: "fontdue rasterization supports sizes up to 65535 pixels".to_string(),
    })?;
    Ok(f32::from(size))
}

fn round_metric(glyph: String, metric: &'static str, value: f32) -> Result<i32, AppError> {
    let rounded = value.round();
    if !rounded.is_finite()
        || f64::from(rounded) > f64::from(i32::MAX)
        || f64::from(rounded) < f64::from(i32::MIN)
    {
        return Err(AppError::MetricOverflow { glyph, metric });
    }
    rounded
        .to_string()
        .parse::<i32>()
        .map_err(|_| AppError::MetricOverflow { glyph, metric })
}

fn format_missing_glyphs(missing_glyphs: &[MissingGlyph]) -> String {
    missing_glyphs
        .iter()
        .map(|missing| format!("{} ({})", missing.display_unit, missing.reason))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ensure(condition: bool, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        if condition {
            Ok(())
        } else {
            Err(std::io::Error::other(message).into())
        }
    }

    #[test]
    fn rejects_multi_scalar_display_units_as_unsupported_clusters()
    -> Result<(), Box<dyn std::error::Error>> {
        let font = load_test_font()?;
        let rasterizer = FontRasterizer { font };

        let result = rasterizer.resolve_char("☀️");

        ensure(
            matches!(result, Err(MissingGlyphReason::UnsupportedCluster)),
            "variation-selector cluster should be unsupported in phase 4",
        )?;
        Ok(())
    }

    #[test]
    fn resolves_single_scalar_when_font_has_glyph() -> Result<(), Box<dyn std::error::Error>> {
        let font = load_test_font()?;
        let rasterizer = FontRasterizer { font };

        let result = rasterizer.resolve_char("A");

        ensure(matches!(result, Ok('A')), "A should resolve")?;
        Ok(())
    }

    fn load_test_font() -> Result<Font, Box<dyn std::error::Error>> {
        let candidates = [
            "/System/Library/Fonts/SFNS.ttf",
            "/System/Library/Fonts/SFNSMono.ttf",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
        ];
        for path in candidates {
            if Path::new(path).is_file() {
                let data = fs::read(path)?;
                return Font::from_bytes(data, FontSettings::default())
                    .map_err(std::io::Error::other)
                    .map_err(Into::into);
            }
        }
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "test font not found").into())
    }
}
