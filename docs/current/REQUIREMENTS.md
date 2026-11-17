# Requirements

## Scope

The project provides a Rust command line tool for generating microcontroller font data from a user-provided character list and a TTF or OTF font file.

Target microcontroller environments:

- ESP32
- Arduino
- Raspberry Pi Pico W

Host execution environments:

- macOS
- Linux

Development environment:

- Rust
- VS Code

## Functional Requirements

### Input

- The user can provide one or more text files containing characters expected to be used by firmware.
- The user can provide a TTF or OTF font file.
- The user can specify one or more font sizes used by the microcontroller display.
- The user can choose C output or Rust output.
- The user can configure generation through a config file.
- The user can override important config values through CLI arguments.

### Character Handling

- The tool must support ASCII letters and symbols.
- The tool must support Japanese hiragana, katakana, and kanji.
- The tool must support emoji sequences to the extent supported by the selected font and rasterization library.
- The tool must remove duplicate display units while preserving stable order.
- The tool must preserve normal spaces only when configured.
- The tool must reject character input that contains no display units after filtering ignored whitespace.
- The tool must report missing glyphs before writing output.
- The tool must report display units that cannot be mapped by the selected rasterization path.

### Font Generation

- The tool must read TTF and OTF fonts.
- The tool must generate bitmap glyphs for all requested font sizes.
- The generated glyph bitmap must use 4-bit grayscale alpha.
- The preferred C output must not require a font renderer or text layout engine on the microcontroller.
- The generated files must be written to the directory where the program is executed unless an explicit output directory is provided.

Phase 4 requirement detail:

- The in-memory font model must be built before output files are generated.
- The model must include size-specific glyph metadata and packed 4-bit bitmap bytes.
- The default missing glyph policy must fail before output generation.
- The skip missing glyph policy must omit missing glyphs while keeping a report.

### Output

- The tool must generate C-compatible font data when requested.
- The tool must generate Rust-compatible font data when requested.
- The output must be deterministic for the same input and config.
- The output must be suitable for embedded projects with constrained storage.
- The command must report which output files were written.
- Generated output must not require heap allocation.
- C fixed bitmap output must expose a direct character mapping and fixed-size bitmap records.
- C fixed bitmap output must be usable with only display-unit lookup, index selection, nibble unpacking, and target display drawing.
- Metrics-based output is allowed only as a separate advanced format and must not be the default microcontroller recommendation.

Phase 8 requirement detail:

- The C fixed output must support a configured cell width and cell height.
- The C fixed output must generate one bitmap record per renderable display unit.
- The C fixed output must preserve input character order.
- The C fixed output must define bytes per character as `(cell_width * cell_height + 1) / 2` for 4-bit alpha data.
- The C fixed output must place every glyph into the fixed cell during generation.
- The C fixed output must not require `advance_x`, `bearing_x`, `bearing_y`, glyph offsets, or variable bitmap lengths at firmware runtime.
- The generated mapping string and bitmap array must remain index-aligned.

Phase 9 documentation requirement detail:

- README must document the complete `c-fixed` workflow from generation through firmware drawing.
- README must identify every generated fixed-cell macro and data symbol used by firmware.
- README must explain UTF-8 display-unit lookup without assuming one byte per character.
- README must explain high-nibble-first 4-bit alpha unpacking.
- README must show how the caller supplies foreground color, background color, and display output.
- README examples must avoid heap allocation.

Phase 10 fixed glyph area requirement detail:

- For `c-fixed` output, the generated drawable glyph area must be smaller than the configured size by one pixel in both dimensions.
- For a configured size `N`, full-width display units must use a generated width of `N - 1`.
- For a configured size `N`, half-width display units must use a generated width of `N / 2 - 1`.
- For a configured size `N`, every generated display unit must use a generated height of `N - 1`.
- ASCII letters, ASCII digits, ASCII punctuation, and ASCII symbols must be treated as half-width display units.
- Non-ASCII display units must be treated as full-width display units.
- For configured size 26, half-width display units must be 12 pixels wide and 25 pixels high.
- For configured size 26, full-width display units must be 25 pixels wide and 25 pixels high.
- Generated C fixed output must expose enough constants for firmware to draw the records without interpreting glyph metrics.
- Generated C fixed output should be close to directly usable in embedded C projects after copying the generated header.

### Verification And Examples

- The repository must include a sample config file.
- The repository must include a sample character file.
- The repository must include a minimal C usage example.
- The repository must include a minimal Rust usage example.
- Documentation must describe integration assumptions for ESP32, Arduino, Raspberry Pi Pico W, and Rust firmware.

## Non-Functional Requirements

- The implementation must avoid hardcoded project-specific fonts or character lists.
- The implementation must keep parsing, rasterization, font modeling, and output generation separated.
- The implementation must avoid over-abstracting before there are multiple real use cases.
- Error messages must identify the file or setting that caused the failure.
- Tests must cover core transformations that affect generated binary data.
- Tests must cover rasterization boundary behavior that determines whether a display unit is renderable.
- Tests must cover deterministic output rendering for C and Rust.
- Tests must cover generated output file writing.
- Tests must cover the Phase 10 half-width and full-width effective cell calculations.

## Phase Approval Requirements

Each phase must stop after its completion criteria are met. The next phase starts only after user approval.

Phase approval points:

- Phase 1 approval: documentation baseline accepted.
- Phase 2 approval: CLI and config foundation accepted.
- Phase 3 approval: character processing accepted.
- Phase 4 approval: rasterization model accepted.
- Phase 5 approval: C and Rust output accepted.
- Phase 6 approval: verification and examples accepted.
- Phase 7 approval: final documentation accepted.
- Phase 8 approval: rendererless C fixed bitmap output accepted.
- Phase 9 approval: C fixed output usage documentation accepted.
- Phase 10 approval: tight C fixed glyph cells accepted.

## Out Of Scope For MVP

- Emitting a full text renderer for a real microcontroller display.
- Generating display driver code.
- Supporting every Unicode code point.
- Guaranteeing emoji rendering when the selected font lacks color or emoji glyphs.
- Font licensing validation.
- GUI or web application.
