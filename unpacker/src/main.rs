#![no_std]
#![no_main]

use memfd_runner::{run_with_options, RunError, RunOptions};

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::CStr;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[unsafe(no_mangle)]
pub extern "C" fn main(argc: isize, argv: *const *const u8) -> isize {
    unsafe {
        run_main(argv_to_vec(argc, argv));
    }

    0
}

fn run_main(args: Vec<String>) {
    let buf = read_self().unwrap();

    if let Some(pos) = find_bytes(&buf, b".packed_elf") {
        let payload = &buf[pos + ".packed_elf".len()..];
        _ = execute(&decode(payload), args).unwrap();
    } else {
        unsafe {
            libc::printf("No payload found.\n\0".as_ptr() as *const _);
        }
    }
}

unsafe fn argv_to_vec(argc: isize, argv: *const *const u8) -> Vec<String> {
    let mut result = Vec::with_capacity(argc as usize);

    for i in 1..argc {
        unsafe {
            let ptr = *argv.offset(i);
            let cstr = CStr::from_ptr(ptr as *const i8);
            let str_slice = cstr.to_str().unwrap_or("<invalid utf8>");
            result.push(String::from(str_slice));
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
                if n <= 0 {
                    break;
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

fn execute(file: &[u8], args: Vec<String>) -> Result<i32, RunError> {
    let arguments = args.iter().map(|s| s.as_ref()).collect::<Vec<_>>();

    let options = RunOptions::new().with_args(&arguments);

    Ok(run_with_options(&file, options)?)
}

fn decode(file: &[u8]) -> Vec<u8> {
    miniz_oxide::inflate::decompress_to_vec(file).unwrap()
}

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { libc::_exit(1) }
}
