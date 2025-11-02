#![no_std]
#![no_main]

#[cfg(feature = "decrypt")]
mod decrypt;
mod fs;
mod parse;

use memfd_runner::{run_with_options, RunError, RunOptions};

extern crate alloc;

use alloc::vec::Vec;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[unsafe(no_mangle)]
pub extern "C" fn main(argc: isize, argv: *const *const u8, envp: *const *const u8) -> isize {
    let mut args = parse::argv_to_vec(&argc, &argv);
    let mut env = parse::env_to_vec(&envp);

    args.truncate(32);
    env.truncate(64);
    run_main(&args, &env);
    0
}

fn run_main(args: &[&str], env: &[&str]) {
    let buf = match fs::read_self() {
        Ok(v) => v,
        Err(_) => unsafe {
            libc::write(1, "Failed to read self\n".as_ptr() as *const _, 20);
            libc::_exit(1);
        },
    };

    if let Some(pos) = find_bytes(&buf, b".packed_elf") {
        #[cfg(feature = "decrypt")]
        let data = decrypt::read_key_and_decrypt(&buf[pos + ".packed_elf".len()..]).unwrap();
        #[cfg(not(feature = "decrypt"))]
        let data = &buf[pos + ".packed_elf".len()..];

        #[allow(clippy::needless_borrow)]
        if execute(&decode(&data), args, env).is_err() {
            unsafe {
                libc::write(1, "Failed to execute payload\n".as_ptr() as *const _, 26);
            }
        }
    } else {
        unsafe {
            libc::write(1, "No payload found.\n".as_ptr() as *const _, 18);
        }
    }
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .rposition(|window| window == needle)
}

fn decode(file: &[u8]) -> Vec<u8> {
    match miniz_oxide::inflate::decompress_to_vec(file) {
        Ok(v) => v,
        Err(_) => unsafe {
            libc::write(1, "Decompression error\n".as_ptr() as *const _, 20);
            libc::_exit(1)
        },
    }
}

fn execute(file: &[u8], args: &[&str], env: &[&str]) -> Result<i32, RunError> {
    let options = RunOptions::new().with_args(args).with_env(env);

    run_with_options(file, options)
}

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { libc::_exit(1) }
}
