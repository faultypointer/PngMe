use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct PngMe {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Encode {
        file: PathBuf,
        chunk_type: String,
        message: String,
        output_file: Option<PathBuf>,
    },
    Decode {
        file: PathBuf,
        chunk_type: String,
    },
    Remove {
        file: PathBuf,
        chunk_type: String,
    },
    Print {
        file: PathBuf,
    },
}
