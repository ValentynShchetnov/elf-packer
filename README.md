# elf-packer

elf-packer is an ELF packer which can be used to obfuscate a binary.

## What is a packer ?

> Binary packers alter the original binary data, and restore it (more or less) before execution.

elf-packer is a very basic packer, encrypting the code section from the binary and decrypting it before executing it. This technique can be used to obfuscate the binary code to bypass antiviruses or to make reverse engineering harder. Packers can also be used to compress a binary to reduce its size.

## Usage

```bash
Usage: packer [OPTIONS] <FILE>

Arguments:
  <FILE>  Path to elf file to be encrypted and packed

Options:
  -k, --key              Enables password-based encryption for the packed file. Prompts for a key if set to true
  -o, --output <OUTPUT>  Optional path for the output packed file. Defaults to the original file's name suffix if not specified. If file already exists adds '.pkd' suffix
  -h, --help             Print help
  -V, --version          Print version
```

## Building

**On Linux**

```bash
	cargo build --release -p unpacker
	cargo build --release -p packer
```

The compiled binary will be available in the **target/release** directory.

## Current Limitations

- **Linux-specific** - requires `memfd_create` system call (Linux 3.17+)
- **Maximum 32 command line arguments** (256 characters each)
- **Maximum 64 environment variables** (256 characters each)
- **No complex ELF features** - no support for dynamic linking validation
