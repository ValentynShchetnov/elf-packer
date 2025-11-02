use alloc::{vec, vec::Vec};

pub fn read_self() -> Result<Vec<u8>, ()> {
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
            read_fd(fd)
        } else {
            read_fd_sized(fd, size)
        }
    }
}

fn read_fd(fd: i32) -> Result<Vec<u8>, ()> {
    unsafe {
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
        Ok(buf)
    }
}

fn read_fd_sized(fd: i32, size: usize) -> Result<Vec<u8>, ()> {
    unsafe {
        let mut buf: Vec<u8> = vec![0; size];

        let r = libc::read(fd, buf.as_mut_ptr() as *mut _, size);

        if r <= 0 {
            return Err(());
        }

        let _ = libc::close(fd);
        Ok(buf)
    }
}
