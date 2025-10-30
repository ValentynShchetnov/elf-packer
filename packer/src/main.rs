use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305,
};
use clap::Parser;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    file: PathBuf,

    #[arg(short, long, default_value = "false")]
    key: bool,

    #[arg(short, long)]
    output: Option<PathBuf>,
}

static UNPACKER: &[u8] = include_bytes!("../../target/release/unpacker");

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let raw_file = fs::read(&args.file)?;

    let key = match args.key {
        true => rpassword::prompt_password("Encryption key: ")?,
        false => "".to_string(),
    };

    #[allow(deprecated)]
    let cipher = ChaCha20Poly1305::new(&Sha256::digest(&key));
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let mut ciphertext = cipher
        .encrypt(&nonce, encode(&raw_file)?.as_slice())
        .unwrap();

    let mut unpacker = UNPACKER.to_vec();
    unpacker.append(&mut b".packed_elf".to_vec());
    unpacker.append(&mut bcrypt::hash(key, 12)?.as_bytes().to_vec());
    unpacker.append(&mut nonce.to_vec());
    unpacker.append(&mut ciphertext);

    fs::write(
        args.output.unwrap_or(args.file.file_name().unwrap().into()),
        &unpacker,
    )?;

    Ok(())
}

fn encode(file: &[u8]) -> anyhow::Result<Vec<u8>> {
    Ok(miniz_oxide::deflate::compress_to_vec(file, 10))
}
