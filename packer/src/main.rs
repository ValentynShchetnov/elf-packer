use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305,
};
use clap::Parser;
use rand::rand_core::{self, TryRngCore};
use sha2::Sha256;
use std::{fs, path::PathBuf};

/// Command-line tool to encrypt and pack ELF-executable files.
/// The tool reads the specified ELF file, applies optional password-based encryption, and writes the packed output to a new file.
/// If no password provided new file acts like original one. Else it asks for a password to decrypt.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Path to elf file to be encrypted and packed
    file: PathBuf,

    /// Enables password-based encryption for the packed file. Prompts for a key if set to true.
    #[arg(short, long, default_value = "false")]
    key: bool,

    /// Optional path for the output packed file. Defaults to the original file's name suffix if not specified.
    /// If file already exists adds '.pkd' suffix.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

include!("include_bin.rs");
const PBKDF2_ROUNDS: u32 = 600_000;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let raw_file = fs::read(&args.file)?;

    let key = match args.key {
        true => rpassword::prompt_password("Encryption key: ")?,
        false => "".to_string(),
    };

    let compressed = compress(&raw_file)?;

    let unpacker = match key.as_bytes() {
        &[] => pack(compressed),
        k => pack_with_key(compressed, k)?,
    };

    let file_name = args.output.unwrap_or_else(|| {
        let mut origin = args.file.file_name().unwrap().to_owned();

        if fs::exists(&origin).unwrap_or(true) {
            origin.push(".pkd");
        }

        origin.into()
    });

    fs::write(&file_name, &unpacker)?;

    let permissions = fs::metadata(&args.file)?.permissions();
    fs::set_permissions(&file_name, permissions)?;

    Ok(())
}

fn compress(file: &[u8]) -> anyhow::Result<Vec<u8>> {
    Ok(miniz_oxide::deflate::compress_to_vec(file, 10))
}

fn pack(mut data: Vec<u8>) -> Vec<u8> {
    let mut unpacker = UNPACKER.to_vec();
    unpacker.append(&mut b".packed_elf".to_vec());
    unpacker.append(&mut data);

    unpacker
}

fn pack_with_key(data: Vec<u8>, key: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut salt = [0; 16];
    let mut buf = [0; 32];
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

    rand_core::OsRng.try_fill_bytes(&mut salt)?;
    pbkdf2::pbkdf2_hmac::<Sha256>(key, &salt, PBKDF2_ROUNDS, &mut buf);

    #[allow(deprecated)]
    let cipher = ChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(&buf));
    let mut ciphertext = cipher.encrypt(&nonce, data.as_slice()).unwrap();

    let mut unpacker = DECRYPT_UNPACKER.to_vec();
    unpacker.append(&mut b".packed_elf".to_vec());
    unpacker.append(&mut salt.to_vec());
    unpacker.append(&mut nonce.to_vec());
    unpacker.append(&mut ciphertext);

    Ok(unpacker)
}
