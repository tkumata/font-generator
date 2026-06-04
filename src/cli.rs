use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "lower")]
pub enum LanguageArg {
    C,
    Rust,
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    #[arg(long)]
    pub font: Option<PathBuf>,

    #[arg(long = "chars")]
    pub chars: Vec<PathBuf>,

    #[arg(long = "size")]
    pub sizes: Vec<u32>,

    #[arg(long)]
    pub language: Option<LanguageArg>,

    #[arg(long = "output-name")]
    pub output_name: Option<String>,

    #[arg(long = "output-dir")]
    pub output_dir: Option<PathBuf>,

    #[arg(long)]
    pub preserve_space: Option<bool>,
}

impl Cli {
    #[must_use]
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
