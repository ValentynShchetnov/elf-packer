use alloc::vec::Vec;
use core::ffi::CStr;

pub fn argv_to_vec<'a>(argc: &'a isize, argv: &'a *const *const u8) -> Vec<&'a str> {
    let mut result = Vec::with_capacity(*argc as usize);

    for i in 1..*argc {
        unsafe {
            let ptr = *argv.offset(i);
            let cstr = CStr::from_ptr(ptr as *const i8);
            let str_slice = cstr.to_str().unwrap_or("");

            if str_slice.len() <= 256 {
                result.push(str_slice);
            }
        }
    }

    result
}

pub fn env_to_vec<'a>(envp: &*const *const u8) -> Vec<&'a str> {
    let mut result = Vec::new();

    unsafe {
        let mut e = *envp;
        while !(*e).is_null() {
            let s = core::ffi::CStr::from_ptr(*e as *const i8);
            let str_slice = s.to_str().unwrap_or("");

            if str_slice.len() <= 256 {
                result.push(str_slice);
            }

            e = e.add(1);
        }
    }

    result
}
