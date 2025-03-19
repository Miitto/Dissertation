use super::{Buffer, GpuBuffer, RawBuffer};

pub struct Mapping<'a> {
    buffer: &'a mut GpuBuffer,
    ptr: *mut std::os::raw::c_void,
    size: usize,
    needs_flush: bool,
    first_written: usize,
    last_written: usize,
}

impl<'a> Mapping<'a> {
    pub fn new(buffer: &'a mut GpuBuffer, loc: *mut std::os::raw::c_void, size: usize) -> Self {
        Self {
            buffer,
            ptr: loc,
            size,
            needs_flush: false,
            first_written: usize::MAX,
            last_written: usize::MIN,
        }
    }

    /// # Safety
    /// src must be a valid pointer for the length of size
    pub unsafe fn write(&mut self, src: *const u8, size: usize, offset: usize) {
        assert!(size + offset <= self.size);

        unsafe { std::ptr::copy_nonoverlapping(src, self.ptr.add(offset) as *mut u8, size) }

        self.needs_flush = true;
        self.first_written = self.first_written.min(offset);
        self.last_written = self.last_written.max(offset + size);
    }
}

impl<'a> Drop for Mapping<'a> {
    fn drop(&mut self) {
        if !self.needs_flush {
            return;
        }

        unsafe {
            gl::FlushMappedNamedBufferRange(
                self.buffer.id(),
                self.first_written as isize,
                (self.last_written - self.first_written) as isize,
            )
        }

        self.buffer.on_map_flush();
    }
}
