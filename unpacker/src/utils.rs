use alloc::vec::Vec;
use memfd_runner::{RunError, RunOptions, run_with_options};

pub fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .rposition(|window| window == needle)
}

pub fn decompress(file: &[u8]) -> Vec<u8> {
    match miniz_oxide::inflate::decompress_to_vec(file) {
        Ok(v) => v,
        Err(_) => unsafe {
            libc::write(1, "Decompression error\n".as_ptr() as *const _, 20);
            libc::_exit(1)
        },
    }
}

pub fn execute(file: &[u8], args: &[&str], env: &[&str]) -> Result<i32, RunError> {
    let options = RunOptions::new().with_args(args).with_env(env);

    run_with_options(file, options)
}
