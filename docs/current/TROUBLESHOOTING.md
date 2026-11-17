# Troubleshooting

## Config File Not Found

Check the path passed to `--config`.

When `--config` is omitted, the tool only auto-loads `fontgen.toml` from the current working directory.

## Font File Not Found

Check `[font].path` or `--font`.

Config paths are resolved relative to the config file location.

## Character File Not Found

Check `[input].chars` or `--chars`.

Multiple `--chars` arguments replace the config list.

## No Characters Found

The input files may contain only ignored whitespace.

Newline, carriage return, and tab are ignored. Normal spaces are ignored unless `preserve_space = true`.

## Missing Glyphs

The selected font does not contain one or more requested glyphs, or the display unit is unsupported by the MVP rasterizer.

Fix options:

- Use a font that contains the requested glyphs.
- Remove unsupported characters from the character file.
- Set `missing_glyphs = "skip"` if omission is acceptable.

## Japanese Glyphs Missing On macOS

Some macOS font collection defaults do not expose all expected Japanese glyphs through the default collection face.

For sample verification on macOS, this font worked in Phase 6:

```sh
/System/Library/Fonts/ヒラギノ角ゴシック W4.ttc
```

## Emoji Does Not Render

The MVP does not support color emoji or multi-scalar emoji sequences.

Variation-selector emoji clusters are reported as unsupported clusters.

## Output Directory Not Found

Create the directory before running the generator:

```sh
mkdir -p generated
```

Then run:

```sh
cargo run -- --config fontgen.toml --output-dir generated
```

## Generated C Needs Platform-Specific Attributes

Add platform-specific memory attributes in the firmware project after generation, or wrap generated data through a project-local adapter.

Examples:

- Arduino `PROGMEM`
- ESP-IDF linker sections
- Pico SDK project-specific const data placement
