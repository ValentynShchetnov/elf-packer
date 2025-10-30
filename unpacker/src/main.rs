#![no_std]
#![no_main]

use chacha20poly1305::{AeadInPlace, ChaCha20Poly1305, KeyInit, Nonce};
use memfd_runner::{run_with_options, RunError, RunOptions};
use sha2::{Digest, Sha256};

extern crate alloc;

use alloc::vec::Vec;
use core::ffi::CStr;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const NONCE_OFFSET: usize = 60;
const DATA_OFFSET: usize = 72;

#[unsafe(no_mangle)]
pub extern "C" fn main(argc: isize, argv: *const *const u8, envp: *const *const u8) -> isize {
    let mut env = Vec::new();

    unsafe {
        let mut e = envp;
        while !(*e).is_null() {
            let s = core::ffi::CStr::from_ptr(*e as *const i8);

            if s.count_bytes() <= 256 {
                env.push(s.to_str().unwrap_or(""));
            }

            e = e.add(1);
        }
    }

    env.truncate(64);
    run_main(&argv_to_vec(&argc, &argv), &env);
    0
}

fn run_main(args: &[&str], env: &[&str]) {
    let buf = match read_self() {
        Ok(v) => v,
        Err(_) => unsafe {
            libc::write(1, "Failed to read self\n".as_ptr() as *const _, 20);
            libc::_exit(1);
        },
    };

    if let Some(pos) = find_bytes(&buf, b".packed_elf") {
        let data = process_payload(&buf[pos + ".packed_elf".len()..]);

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

fn process_payload(payload: &[u8]) -> Vec<u8> {
    let hash = match str::from_utf8(&payload[..NONCE_OFFSET]) {
        Ok(v) => v,
        Err(_) => unsafe {
            libc::write(1, "Failed to read metadata\n".as_ptr() as *const _, 24);
            libc::_exit(1)
        },
    };
    let nonce = &payload[NONCE_OFFSET..DATA_OFFSET];

    let mut key = [0; 256];

    let len = match bcrypt::verify("", hash) {
        Ok(true) => 0,
        Ok(false) => read_key(&mut key),
        Err(_) => unsafe {
            libc::write(1, "Bcrypt error\n".as_ptr() as *const _, 13);
            libc::_exit(1)
        },
    };

    let mut result = payload[DATA_OFFSET..].to_vec();
    match bcrypt::verify(&key[..len], hash) {
        Ok(true) => {
            let cipher = ChaCha20Poly1305::new(&Sha256::digest(&key[..len]));

            #[allow(deprecated)]
            if cipher
                .decrypt_in_place(Nonce::from_slice(&nonce), b"", &mut result)
                .is_err()
            {
                unsafe {
                    libc::write(1, "Decryptor error\n".as_ptr() as *const _, 16);
                    libc::_exit(1)
                }
            }

            result
        }
        Ok(false) => unsafe {
            libc::write(1, "Wrong key\n".as_ptr() as *const _, 10);
            libc::_exit(1)
        },
        Err(_) => unsafe {
            libc::write(1, "Bcrypt error\n".as_ptr() as *const _, 13);
            libc::_exit(1)
        },
    }
}

fn read_key(buf: &mut [u8]) -> usize {
    unsafe {
        libc::write(1, "Decryption key: ".as_ptr() as *const _, 16);

        let mut term: libc::termios = core::mem::zeroed();
        libc::tcgetattr(0, &mut term);
        let old_term = term;
        term.c_lflag &= !libc::ECHO;
        libc::tcsetattr(0, libc::TCSANOW, &term);

        let mut total = 0;
        while total < buf.len() {
            let mut byte: u8 = 0;
            let r = libc::read(0, &mut byte as *mut u8 as *mut _, 1);
            if r <= 0 {
                break;
            }

            if byte == b'\n' || byte == b'\r' {
                break;
            }

            buf[total] = byte;
            total += 1;
        }

        libc::tcsetattr(0, libc::TCSANOW, &old_term);

        libc::write(1, "\n".as_ptr() as *const _, 1);
        total
    }
}

fn argv_to_vec<'a>(argc: &'a isize, argv: &'a *const *const u8) -> Vec<&'a str> {
    let mut result = Vec::with_capacity(*argc as usize);

    for i in 1..*argc {
        unsafe {
            let ptr = *argv.offset(i);
            let cstr = CStr::from_ptr(ptr as *const i8);
            let str_slice = cstr.to_str().unwrap_or("");
            result.push(str_slice); // String::from(str_slice));
        }
    }

    result
}

fn read_self() -> Result<Vec<u8>, ()> {
    unsafe {
        let path = b"/proc/self/exe\0";
        let fd = libc::open(path.as_ptr() as *const _, 0);
        if fd < 0 {
            return Err(());
        }

        let mut stats: libc::stat = core::mem::zeroed();
        if libc::fstat(fd, &mut stats) != 0 {
            let _ = libc::close(fd);
            return Err(());
        }
        let size = stats.st_size as usize;

        if size == 0 {
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                let n = libc::read(fd, chunk.as_mut_ptr() as *mut _, chunk.len());
                if n == 0 {
                    break;
                } else if n < 0 {
                    let _ = libc::close(fd);
                    return Err(());
                }
                let n = n as usize;
                buf.extend_from_slice(&chunk[..n]);
            }
            let _ = libc::close(fd);
            return Ok(buf);
        } else {
            let mut buf: Vec<u8> = Vec::with_capacity(size);
            buf.set_len(size);

            let r = libc::read(fd, buf.as_mut_ptr() as *mut _, size);

            if r <= 0 {
                return Err(());
            }

            let _ = libc::close(fd);
            return Ok(buf);
        }
    }
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .rposition(|window| window == needle)
}

fn execute(file: &[u8], args: &[&str], env: &[&str]) -> Result<i32, RunError> {
    let options = RunOptions::new().with_args(&args).with_env(&env);

    Ok(run_with_options(&file, options)?)
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

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { libc::_exit(1) }
}
