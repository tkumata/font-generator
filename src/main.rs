use font_generator::chars::CharacterSet;
use font_generator::cli::Cli;
use font_generator::config::GenerationSettings;
use font_generator::font::generate_font_model;
use font_generator::output::{format_written_files, write_output};

fn main() {
    let cli = Cli::parse_args();

    match GenerationSettings::from_cli(&cli) {
        Ok(settings) => {
            print!("{}", settings.format_normalized());
            match CharacterSet::from_settings(&settings) {
                Ok(characters) => {
                    print!("{}", characters.format_summary());
                    match generate_font_model(&settings, &characters) {
                        Ok(font) => {
                            print!("{}", font.format_summary());
                            match write_output(&settings, &font) {
                                Ok(paths) => {
                                    print!("{}", format_written_files(&paths));
                                }
                                Err(error) => {
                                    eprintln!("error: {error}");
                                    std::process::exit(1);
                                }
                            }
                        }
                        Err(error) => {
                            eprintln!("error: {error}");
                            std::process::exit(1);
                        }
                    }
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    std::process::exit(1);
                }
            }
        }
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}
