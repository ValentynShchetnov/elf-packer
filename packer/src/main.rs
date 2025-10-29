use clap::Parser;
use std::{fs, path::PathBuf};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    file: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,
}

static UNPACKER: &[u8] = include_bytes!("../../target/release/unpacker");

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let raw_file = fs::read(&args.file)?;

    let mut unpacker = UNPACKER.to_vec();
    unpacker.append(&mut b".packed_elf".to_vec());
    unpacker.append(&mut encode(&raw_file)?);

    fs::write(
        args.output.unwrap_or(args.file.file_name().unwrap().into()),
        &unpacker,
    )?;

    Ok(())
}

fn encode(file: &[u8]) -> anyhow::Result<Vec<u8>> {
    Ok(miniz_oxide::deflate::compress_to_vec(file, 10))
}
