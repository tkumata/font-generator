# Font Generator Development Plan

## Purpose

Build a Rust command line tool that reads a project-specific character list and a TTF or OTF font file, then generates compact microcontroller font data for ESP32, Arduino, and Raspberry Pi Pico W projects.

The generated font data uses 4-bit grayscale alpha bitmaps so embedded display code can render pseudo-antialiased text without storing full 8-bit alpha glyphs.

## Development Rules

- Documentation is created before implementation.
- Implementation starts only after user approval of the documentation.
- Source changes and documentation must stay synchronized.
- Each phase must be independently understandable by a future agent.
- Each phase ends with a user approval gate before the next phase starts.
- No destructive git command may be used without explicit user instruction.

## Configuration Direction

Font generation should use a configuration file plus separate character list files.

Recommended files:

- `fontgen.toml`: generation settings such as source font, sizes, output language, output name, and input character files.
- `chars.txt`: characters used by the target firmware or display UI.
- Optional grouped files such as `chars/weather.txt`, `chars/ui.txt`, and `chars/messages.txt`.

This keeps reusable character lists separate from generation options. CLI arguments may override configuration values for one-off builds.

## Phases

### Phase 1: Documentation Baseline

Create the initial project documentation.

Deliverables:

- `docs/PLAN.md`
- `docs/REQUIREMENTS.md`
- `docs/SPECIFICATIONS.md`
- `docs/DESIGN.md`
- `docs/ADR/202606041017.md`
- `docs/TASK/current-task.md`
- `README.md`

Completion criteria:

- The tool scope is clear.
- Phase boundaries are clear.
- User approval gates are documented.
- No source implementation is added.

Approval gate:

Wait for user approval before Phase 2.

### Phase 2: Rust CLI Foundation

Create the Rust project and command line structure.

Deliverables:

- Rust crate scaffold.
- CLI options for config path, font path, character files, sizes, output language, output name, and output directory.
- Config loading from `fontgen.toml`.
- Validation errors for missing files and invalid settings.
- VS Code development notes.

Completion criteria:

- The CLI can parse configuration and print normalized generation settings.
- No font rasterization is required yet.
- Unit tests cover config parsing and CLI overrides.

Approval gate:

Wait for user approval before Phase 3.

### Phase 3: Character Input Processing

Implement Unicode-aware character collection.

Deliverables:

- Read one or more text files.
- Collect unique display units in stable order.
- Support Japanese text, ASCII symbols, and emoji sequences.
- Preserve normal spaces when configured.
- Report unsupported or invalid input clearly.

Completion criteria:

- Duplicate characters are removed.
- Output order is deterministic.
- Tests cover ASCII, Japanese, symbols, emoji variation selectors, and whitespace handling.

Approval gate:

Wait for user approval before Phase 4.

### Phase 4: Font Loading And Rasterization

Load TTF or OTF files and rasterize glyphs for one or more sizes.

Deliverables:

- Font file loading.
- Glyph lookup.
- Per-size rasterization.
- 8-bit coverage to 4-bit alpha conversion.
- Glyph metrics capture.
- Missing glyph reporting.

Completion criteria:

- Given a font, character list, and size list, the tool creates an in-memory font model.
- Tests cover alpha quantization and glyph metrics normalization.
- Missing or unsupported display units are reported according to the configured policy.

Approval gate:

Wait for user approval before Phase 5.

### Phase 5: C And Rust Output Generation

Generate firmware-consumable font data.

Deliverables:

- C header/source output.
- Rust module output.
- Size-specific font tables.
- Packed 4-bit alpha bitmap arrays.
- Glyph metadata tables.
- Lookup helpers or lookup data suitable for embedded code.

Completion criteria:

- C output can be included from embedded C or Arduino code.
- Rust output can be imported as a module.
- Tests cover deterministic output and packed bitmap data.
- The CLI writes the configured output files and reports their paths.

Approval gate:

Wait for user approval before Phase 6.

### Phase 6: Verification And Examples

Add verification assets and sample usage.

Deliverables:

- Sample `fontgen.toml`.
- Sample `chars.txt`.
- Generated-output tests.
- Minimal C usage example.
- Minimal Rust usage example.
- Documentation for ESP32, Arduino, and Pico W integration assumptions.

Completion criteria:

- A new user can run a sample generation command.
- Test output is deterministic.
- Examples show how to access glyph metadata and bitmap bytes.
- Integration assumptions for ESP32, Arduino, Pico W, and Rust firmware are documented.

Approval gate:

Wait for user approval before Phase 7.

### Phase 7: Final Documentation And Operational Polish

Finalize README and operational docs.

Deliverables:

- Complete setup instructions for Mac and Linux.
- VS Code setup.
- CLI reference.
- Configuration reference.
- Known limitations.
- Troubleshooting notes.

Completion criteria:

- Documentation matches the implemented behavior.
- The project is ready for normal use in embedded display projects.
- Setup, CLI, configuration, limitations, and troubleshooting docs are available.

Approval gate:

Wait for user approval before any post-MVP expansion.

### Phase 8: Rendererless C Fixed Bitmap Output

Correct the C output direction for typical microcontroller firmware.

Firmware projects often do not have a font renderer, a text layout engine, or baseline-aware glyph placement code. The C output must therefore provide font data that can be consumed with a small bitmap drawing loop.

Deliverables:

- Add a rendererless C output mode for fixed-cell bitmap fonts.
- Support configured cell width and cell height for C fixed output.
- Generate one fixed-length bitmap record per display unit.
- Preserve character-list order so character index and bitmap index are identical.
- Keep 4-bit alpha nibble packing.
- Generate C symbols and macros that expose width, height, bits per pixel, bytes per character, character count, character mapping, and bitmap data.
- Document that metrics-based C output is not the preferred microcontroller path.
- Update C example code to demonstrate direct fixed-cell bitmap lookup and nibble expansion.

Completion criteria:

- C fixed output can be used without a font renderer.
- Firmware needs only UTF-8 display-unit lookup, bitmap index selection, and a pixel or rectangle drawing loop.
- Generated C fixed data uses deterministic fixed-size records.
- `make check` passes.
- `make build` passes.

Approval gate:

Wait for user approval after Phase 8 documentation before implementation.

## Deferred Topics

- Font subsetting for proportional text layout engines.
- Compression beyond 4-bit alpha packing.
- Runtime kerning tables.
- Bitmap preview UI.
- Web UI.
