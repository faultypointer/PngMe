use std::{fs, str::FromStr};

use args::{Commands, PngMe};
use chunk::Chunk;
use chunk_type::ChunkType;
use clap::Parser;
use png::Png;

mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

pub type Error = anyhow::Error;
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let pngme = PngMe::parse();

    match &pngme.command {
        Commands::Encode {
            file,
            chunk_type,
            message,
            output_file,
        } => {
            let chunk_type = ChunkType::from_str(chunk_type)?;
            let bytes = fs::read(file)?;

            let mut png = Png::try_from(bytes.as_ref())?;
            let chunk = Chunk::new(chunk_type, message.clone().into_bytes());
            png.append_chunk(chunk);
            if let Some(op_file) = output_file {
                fs::write(op_file, png.as_bytes())?;
            } else {
                fs::write(file, png.as_bytes())?;
            }
        }
        Commands::Decode { file, chunk_type } => {
            let bytes = fs::read(file)?;
            let png = Png::try_from(bytes.as_ref())?;

            if let Some(chunk) = png.chunk_by_type(chunk_type) {
                println!("Decoded Message: {}", chunk.data_as_string()?);
            }
        }
        Commands::Remove { file, chunk_type } => {
            let bytes = fs::read(file)?;
            let mut png = Png::try_from(bytes.as_ref())?;

            png.remove_first_chunk(chunk_type)?;
            fs::write(file, png.as_bytes())?;
        }
        Commands::Print { file } => {
            let bytes = fs::read(file)?;
            let png = Png::try_from(bytes.as_ref())?;

            println!("{}", png);
        }
    }

    Ok(())
}
