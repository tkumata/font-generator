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
- The generated data must include enough metrics for firmware rendering.
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
- The C and Rust outputs must expose packed bitmap bytes and glyph metadata without requiring heap allocation.

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

## Out Of Scope For MVP

- Rendering text on a real microcontroller display.
- Generating display driver code.
- Supporting every Unicode code point.
- Guaranteeing emoji rendering when the selected font lacks color or emoji glyphs.
- Font licensing validation.
- GUI or web application.
