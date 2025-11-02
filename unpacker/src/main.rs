#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(test, feature = "decrypt"))]
mod decrypt;
#[cfg(not(test))]
mod fs;
#[cfg(not(test))]
mod parse;
#[cfg(not(test))]
mod utils;

extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn main(argc: isize, argv: *const *const u8, envp: *const *const u8) -> isize {
    let mut args = parse::argv_to_vec(&argc, &argv);
    let mut env = parse::env_to_vec(&envp);

    args.truncate(32);
    env.truncate(64);
    run_main(&args, &env);
    0
}

#[cfg(not(test))]
fn run_main(args: &[&str], env: &[&str]) {
    let buf = match fs::read_self() {
        Ok(v) => v,
        Err(_) => unsafe {
            libc::write(1, "Failed to read self\n".as_ptr() as *const _, 20);
            libc::_exit(1);
        },
    };

    if let Some(pos) = utils::find_bytes(&buf, b".packed_elf") {
        #[cfg(feature = "decrypt")]
        let data = decrypt::read_key_and_decrypt(&buf[pos + ".packed_elf".len()..]).unwrap();
        #[cfg(not(feature = "decrypt"))]
        let data = &buf[pos + ".packed_elf".len()..];

        #[allow(clippy::needless_borrow)]
        if utils::execute(&utils::decompress(&data), args, env).is_err() {
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

#[cfg(not(test))]
#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { libc::_exit(1) }
}
