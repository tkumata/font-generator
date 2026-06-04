# CLI Reference

## Basic Command

Use a config file:

```sh
cargo run -- --config fontgen.toml
```

Use only CLI arguments:

```sh
cargo run -- \
  --font ./fonts/NotoSansJP-Regular.ttf \
  --chars ./chars.txt \
  --size 16 \
  --language c \
  --output-name app_font \
  --output-dir ./generated
```

## Options

`--config <path>`

Path to a TOML configuration file. When omitted, `fontgen.toml` in the current directory is used if it exists.

`--font <path>`

Overrides `[font].path`.

`--chars <path>`

Overrides `[input].chars`. May be passed multiple times.

`--size <pixels>`

Overrides `[generation].sizes`. May be passed multiple times.

`--language <c|rust>`

Overrides `[output].language`.

`--output-name <name>`

Overrides `[output].name`. The name must start with an ASCII letter or underscore and contain only ASCII letters, digits, or underscores.

`--output-dir <path>`

Overrides `[output].directory`. The directory must already exist.

`--preserve-space <true|false>`

Overrides `[input].preserve_space`.

## Config-Only Output Settings

`[output].format`

Selects the C output format.

Supported values:

- `c-fixed`: fixed-cell C bitmap header for firmware without a font renderer.
- `c-metrics`: metrics-based C header/source output for advanced firmware that owns renderer logic.

When omitted, compatibility C output uses the metrics format.

`[fixed_cell].width`

Fixed output cell width in pixels for `output.format = "c-fixed"`.

`[fixed_cell].height`

Fixed output cell height in pixels for `output.format = "c-fixed"`.

## Command Output

The command prints:

- Normalized settings.
- Character file paths.
- Ordered collected display units.
- Per-size generated glyph counts and bitmap byte counts.
- Missing glyph report.
- Written output file paths.

## Exit Behavior

The command exits with a non-zero status when:

- Required settings are missing.
- A configured file does not exist.
- A character file cannot be read as UTF-8.
- No display units remain after whitespace filtering.
- The font cannot be parsed.
- A requested glyph is missing while `missing_glyphs = "error"`.
- Fixed-cell C output is selected without exactly one size.
- Fixed-cell C output is selected without `[fixed_cell]` settings.
- A glyph bitmap does not fit the configured fixed cell.
- An output file cannot be written.
