use std::fs;
use std::path::{Path, PathBuf};

use font_generator::config::{
    FixedCellSettings, GenerationSettings, MissingGlyphPolicy, OutputFormat, OutputLanguage,
};
use font_generator::model::{GeneratedFont, GeneratedFontSize, Glyph};
use font_generator::output::write_output;

fn sample_font() -> GeneratedFont {
    GeneratedFont {
        name: "sample_font".to_string(),
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
    }
}

fn settings_for(language: OutputLanguage, output_directory: PathBuf) -> GenerationSettings {
    GenerationSettings {
        config_path: None,
        font_path: PathBuf::from("unused.ttf"),
        character_files: vec![PathBuf::from("unused.txt")],
        preserve_space: false,
        language,
        output_format: None,
        output_name: "sample_font".to_string(),
        output_directory,
        sizes: vec![16],
        alpha_bits: 4,
        missing_glyphs: MissingGlyphPolicy::Error,
        fixed_cell: None,
    }
}

fn fixed_settings(output_directory: PathBuf) -> GenerationSettings {
    GenerationSettings {
        config_path: None,
        font_path: PathBuf::from("unused.ttf"),
        character_files: vec![PathBuf::from("unused.txt")],
        preserve_space: false,
        language: OutputLanguage::C,
        output_format: Some(OutputFormat::CFixed),
        output_name: "sample_font".to_string(),
        output_directory,
        sizes: vec![16],
        alpha_bits: 4,
        missing_glyphs: MissingGlyphPolicy::Error,
        fixed_cell: Some(FixedCellSettings {
            width: 4,
            height: 4,
        }),
    }
}

fn temp_dir(test_name: &str) -> Result<PathBuf, std::io::Error> {
    let root = std::env::temp_dir().join(format!(
        "font-generator-generated-output-{test_name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root)?;
    Ok(root)
}

fn ensure_contains(path: &Path, needle: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    if content.contains(needle) {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "{} did not contain expected text: {needle}",
            path.display()
        ))
        .into())
    }
}

#[test]
fn writes_deterministic_c_output_files() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("c")?;
    let settings = settings_for(OutputLanguage::C, root.clone());

    let paths = write_output(&settings, &sample_font())?;

    if paths.len() != 2 {
        return Err(std::io::Error::other(format!("unexpected paths: {paths:?}")).into());
    }
    ensure_contains(&root.join("sample_font.h"), "typedef struct")?;
    ensure_contains(&root.join("sample_font.c"), "0xf0, 0x81")?;
    ensure_contains(
        &root.join("sample_font.c"),
        "{ \"A\", 0u, 2u, 2u, 2u, 3, 0, -1 }",
    )?;
    Ok(())
}

#[test]
fn writes_deterministic_fixed_c_output_file() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("fixed-c")?;
    let settings = fixed_settings(root.clone());

    let paths = write_output(&settings, &sample_font())?;

    if paths.len() != 1 {
        return Err(std::io::Error::other(format!("unexpected paths: {paths:?}")).into());
    }
    ensure_contains(&root.join("sample_font.h"), "#define SAMPLE_FONT_WIDTH 3")?;
    ensure_contains(&root.join("sample_font.h"), "#define SAMPLE_FONT_HEIGHT 3")?;
    ensure_contains(
        &root.join("sample_font.h"),
        "#define SAMPLE_FONT_FULL_WIDTH 3",
    )?;
    ensure_contains(
        &root.join("sample_font.h"),
        "#define SAMPLE_FONT_HALF_WIDTH 1",
    )?;
    ensure_contains(
        &root.join("sample_font.h"),
        "#define SAMPLE_FONT_FULL_BYTES_PER_CHAR 5",
    )?;
    ensure_contains(
        &root.join("sample_font.h"),
        "#define SAMPLE_FONT_HALF_BYTES_PER_CHAR 2",
    )?;
    ensure_contains(
        &root.join("sample_font.h"),
        "#define SAMPLE_FONT_BYTES_PER_CHAR 5",
    )?;
    ensure_contains(
        &root.join("sample_font.h"),
        "static const char *sample_font_chars",
    )?;
    ensure_contains(&root.join("sample_font.h"), "\"A\";")?;
    ensure_contains(
        &root.join("sample_font.h"),
        "static const uint8_t sample_font_widths[1]",
    )?;
    ensure_contains(&root.join("sample_font.h"), "    1u, // 'A'")?;
    ensure_contains(
        &root.join("sample_font.h"),
        "static const uint8_t sample_font_data[1][5]",
    )?;
    Ok(())
}

#[test]
fn writes_deterministic_rust_output_file() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("rust")?;
    let settings = settings_for(OutputLanguage::Rust, root.clone());

    let paths = write_output(&settings, &sample_font())?;

    if paths.len() != 1 {
        return Err(std::io::Error::other(format!("unexpected paths: {paths:?}")).into());
    }
    ensure_contains(
        &root.join("sample_font.rs"),
        "pub const SIZE_16_BITMAP_DATA",
    )?;
    ensure_contains(&root.join("sample_font.rs"), "0xf0, 0x81")?;
    ensure_contains(&root.join("sample_font.rs"), "Glyph { key: \"A\"")?;
    Ok(())
}
