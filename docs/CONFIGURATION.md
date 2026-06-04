# Configuration Reference

## Example

```toml
[font]
path = "./fonts/NotoSansJP-Regular.ttf"

[input]
chars = ["./chars.txt"]
preserve_space = true

[output]
language = "c"
name = "app_font"
directory = "."

[generation]
sizes = [16, 24]
alpha_bits = 4
missing_glyphs = "error"
```

## Path Resolution

Paths from the configuration file are resolved relative to the configuration file location.

CLI override paths are currently resolved through the same normalization path as config values.

The output directory must already exist.

## `[font]`

`path`

Path to a TTF, OTF, or supported font collection file. The selected font must contain every requested glyph when `missing_glyphs = "error"`.

## `[input]`

`chars`

List of character input files. Files are read in the configured order.

`preserve_space`

When true, normal spaces are included as display units. Newline, carriage return, and tab are ignored.

## `[output]`

`language`

Supported values:

- `c`
- `rust`

`name`

Base name for generated files and C symbols.

Rules:

- Must start with an ASCII letter or underscore.
- Must contain only ASCII letters, digits, or underscores.

`directory`

Directory where generated files are written.

## `[generation]`

`sizes`

One or more positive integer pixel sizes.

`alpha_bits`

Only `4` is supported in the MVP.

`missing_glyphs`

Supported values:

- `error`: fail when any requested display unit cannot be rendered.
- `skip`: omit unsupported or missing glyphs and report them.

## Character Collection Rules

- Input files must be UTF-8.
- Display units are collected as Unicode grapheme clusters.
- Duplicate display units are removed.
- First occurrence determines output order.
- Newline, carriage return, and tab are ignored.
- Normal spaces are included only when `preserve_space = true`.

## Rasterization Limits

The MVP rasterizer uses `fontdue` and renders single Unicode scalar display units.

Multi-scalar grapheme clusters, including variation-selector emoji sequences, are reported as unsupported clusters.
