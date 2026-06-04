# Font Generator

Rust command line tool for generating compact microcontroller font data from project-specific character lists and TTF or OTF fonts.

Target firmware environments:

- ESP32
- Arduino
- Raspberry Pi Pico W

Host environments:

- macOS
- Linux

## Goal

Embedded display projects often need only a small subset of Japanese text, symbols, and emoji. Storing a complete font is usually too expensive for microcontrollers, so this project generates only the glyphs that firmware expects to use.

Generated glyph bitmaps use 4-bit grayscale alpha for pseudo-antialiased rendering.

## Usage

```sh
cargo run -- --config fontgen.toml
cargo run -- --font ./fonts/NotoSansJP-Regular.ttf --chars ./chars.txt --size 20 --language rust
```

Example `fontgen.toml`:

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

Useful overrides:

```sh
cargo run -- --config fontgen.toml --size 16 --size 24 --language c
```

The command prints normalized generation settings, reads configured character files, prints the ordered unique display units, loads the configured font, builds an in-memory rasterized font model, and writes output files for the configured language.

Supported options:

- `--config`: path to the TOML config file. When omitted, `fontgen.toml` is loaded if it exists; otherwise required settings must be provided through CLI arguments.
- `--font`: override `[font].path`.
- `--chars`: override `[input].chars`. May be passed multiple times.
- `--size`: override `[generation].sizes`. May be passed multiple times.
- `--language`: override `[output].language`. Supported values are `c` and `rust`.
- `--output-name`: override `[output].name`.
- `--output-dir`: override `[output].directory`.
- `--preserve-space`: override `[input].preserve_space` with `true` or `false`.

Example `chars.txt`:

```text
abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ
!@#$%^&*()-_=+[]{}\;:'",<.>/?℃
☀️☁️☔️🌤️⛅️⛄️
あいうえおかきくけこ
湿度温度気圧設定空白削除漢字
```

## Font Rasterization

Phase 4 uses `fontdue` for TTF and OTF loading and grayscale rasterization.

Rasterization behavior:

- Single-scalar display units are looked up and rasterized through the selected font.
- Multi-scalar grapheme clusters, including variation-selector emoji clusters, are reported as unsupported clusters.
- `missing_glyphs = "error"` stops generation when a requested display unit cannot be rendered.
- `missing_glyphs = "skip"` omits unsupported or missing display units and reports them in the model summary.
- 8-bit coverage is quantized to 4-bit alpha with rounding.
- Two 4-bit alpha pixels are packed into each byte in the in-memory model.

## Output Generation

When `language = "c"`, the tool writes:

- `{output_name}.h`
- `{output_name}.c`

When `language = "rust"`, the tool writes:

- `{output_name}.rs`

The generated files contain size-specific packed bitmap byte arrays, glyph metadata arrays, and a top-level size table. The command prints every written path.

## Examples

Sample inputs:

- `examples/fontgen.toml`
- `examples/chars.txt`

Minimal usage examples:

- `examples/c/minimal_usage.c`
- `examples/rust/minimal_usage.rs`

Integration notes:

- `docs/INTEGRATION.md`

## Operational Documentation

- `docs/SETUP.md`: macOS, Linux, and VS Code setup.
- `docs/CLI.md`: command options and exit behavior.
- `docs/CONFIGURATION.md`: TOML settings and path rules.
- `docs/LIMITATIONS.md`: MVP limits.
- `docs/TROUBLESHOOTING.md`: common failures and fixes.

## Development Status

MVP Phase 7 documentation polish is implemented. The tool can load configuration, apply CLI overrides, validate required files and settings, collect Unicode grapheme clusters from character files, remove duplicates in stable order, load a configured font, rasterize scalar glyphs for configured sizes, print an in-memory font model summary, write generated output files, and provide sample integration materials.

Development phases:

- Phase 1: Documentation baseline.
- Phase 2: Rust CLI foundation.
- Phase 3: Character input processing.
- Phase 4: Font loading and rasterization.
- Phase 5: C and Rust output generation.
- Phase 6: Verification and examples.
- Phase 7: Final documentation and operational polish.

The planned MVP phases are complete after Phase 7 approval.

## Development

Recommended VS Code extensions:

- `rust-lang.rust-analyzer`
- `tamasfe.even-better-toml`

Useful commands:

```sh
cargo fmt
cargo test
cargo run -- --help
make check
make build
```

`make check` runs formatting, Clippy with denied warnings, and tests. `make build` must also pass before a phase is treated as complete.

The project currently uses a normal Cargo binary crate. No VS Code workspace-specific configuration is required.

## Documentation

- `docs/PLAN.md`
- `docs/REQUIREMENTS.md`
- `docs/SPECIFICATIONS.md`
- `docs/DESIGN.md`
- `docs/INTEGRATION.md`
- `docs/SETUP.md`
- `docs/CLI.md`
- `docs/CONFIGURATION.md`
- `docs/LIMITATIONS.md`
- `docs/TROUBLESHOOTING.md`
- `docs/ADR/202606041017.md`
- `docs/ADR/202606041204.md`
- `docs/ADR/202606041411.md`
- `docs/ADR/202606041418.md`
- `docs/TASK/current-task.md`
