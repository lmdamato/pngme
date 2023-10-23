use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use anyhow::anyhow;
use clap::Parser;
use crate::chunk::{Chunk, IChunk};
use crate::chunk_type::ChunkType;
use crate::commands::Command;
use crate::png::{IPng, Png};


mod chunk;
mod chunk_type;
mod commands;
mod png;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}


fn read_png_from_file(path: &PathBuf) -> anyhow::Result<Png> {
    fs::read(path)
        .map_err(|e| anyhow!(e))
        .and_then(|fin| Png::try_from(fin.as_slice()))
}


fn write_png(path: &Path, png: &Png) -> anyhow::Result<()> {
    fs::write(path, png.as_bytes())
        .map_err(|e| anyhow!(e))
}


fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(c) => match c {
            Command::Encode { path, chunk_type, message} => {
                let chunk_type_obj = ChunkType::from_str(chunk_type)?;
                let chunk = Chunk::new(chunk_type_obj, message.as_bytes().to_vec());
                let mut png = read_png_from_file(path)?;

                png.append_chunk(chunk);
                write_png(path, &png)?;

                println!("Successfully encoded message in PNG file.")
            },
            Command::Decode { path, chunk_type } => {
                let mut png = read_png_from_file(path)?;
                let chunk = png.remove_chunk(chunk_type)?;

                println!("Found! Message:\n\t{}", chunk.data_as_string()?);
            }
            Command::Remove { path, chunk_type } => {
                let mut png = read_png_from_file(path)?;
                let _chunk = png.remove_chunk(chunk_type)?;

                write_png(path, &png)?;

                println!("Successfully removed chunk from PNG file.")
            }
            Command::Print { path } => {
                let png = read_png_from_file(path)?;
                println!("{}", png);
            }
        },
        _ => return Err(anyhow!("No command specified")),
    }

    Ok(())
}
